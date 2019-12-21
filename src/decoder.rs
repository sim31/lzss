use super::*;
use bitbit::{BitReader, MSB};
use slice_deque::SliceDeque;
use std::io::{Error, ErrorKind, Read, Result, Write};

// TODO: Make encoder use this too?
enum Record {
    Reference { position: usize, length: usize },
    Literal { byte: u8 },
}

pub struct Decoder<R: Read, W: Write> {
    br: BitReader<R, MSB>,
    writer: W,
    history_addr_nbits: usize,
    match_length_nbits: usize,
    history: SliceDeque<u8>,
    history_size: usize,
    current_window_size: usize,
}

impl<R: Read, W: Write> Decoder<R, W> {
    pub fn new(reader: R, writer: W) -> Result<Decoder<R, W>> {
        let mut br: BitReader<_, MSB> = BitReader::new(reader);

        let (history_addr_nbits, match_length_nbits) = Decoder::<R, W>::read_header(&mut br)?;
        assert!(
            history_addr_nbits >= MIN_HISTORY_ADDR_BITS
                && history_addr_nbits <= MAX_HISTORY_ADDR_BITS
        );
        assert!(
            match_length_nbits >= MIN_MATCH_LENGTH_BITS
                && match_length_nbits <= MAX_MATCH_LENGTH_BITS
        );

        let history_size: usize = usize::pow(2, history_addr_nbits as u32);
        let threshold = calc_threshold(history_addr_nbits, match_length_nbits);
        let current_window_size = usize::pow(2, match_length_nbits as u32) + threshold;

        Ok(Decoder {
            br,
            writer,
            history_addr_nbits,
            match_length_nbits,
            history: SliceDeque::<u8>::with_capacity(history_size),
            history_size,
            current_window_size,
        })
    }

    pub fn decode(&mut self) -> Result<()> {
        // Read beginning of a file
        self.init()?;

        // Decode rest of a file
        loop {
            if let Some(record) = self.read_next_record()? {
                self.write_decoded(&record)?;
            } else {
                // None returned from read_next_record means file has ended
                return Ok(());
            }
        }
    }

    // Returns None if file ends
    // Else the next Record if or an error
    #[allow(clippy::match_bool)]
    fn read_next_record(&mut self) -> Result<Option<Record>> {
        // There are two valid ways for an archive file to end:
        //  1. At the byte boundary (if the end of the last record is at the byte boundary)
        //  2. Or if the end of the last record is in the middle of the byte,
        //  it has to end with a 1 and padding till the next byte boundary
        //  (which creates an invalid literal record - that's how we know it's the end).
        // If we get EOF when reading type bit, it's the first type of ending.
        // If we get a literal type bit and EOF while reading it's byte, it means it's the second type of ending.
        // Every other case of EOF is interpreted as InvalidData error.

        match self.br.read_bit() {
            Ok(rec_type_bit) => match rec_type_bit {
                RECORD_TYPE_LITERAL => self.read_literal(),
                RECORD_TYPE_REFERENCE => self.read_reference(),
            },
            Err(error) => match error.kind() {
                ErrorKind::UnexpectedEof => Ok(None),
                _ => Err(error),
            },
        }
    }

    fn read_literal(&mut self) -> Result<Option<Record>> {
        match self.br.read_byte() {
            Ok(byte) => Ok(Some(Record::Literal { byte })),
            Err(error) => match error.kind() {
                ErrorKind::UnexpectedEof => Ok(None),
                _ => Err(error),
            },
        }
    }

    fn read_reference(&mut self) -> Result<Option<Record>> {
        let position = self.br.read_bits(self.history_addr_nbits)? as usize;
        let length = self.br.read_bits(self.match_length_nbits)? as usize;
        Ok(Some(Record::Reference { position, length }))
    }

    // Read unencoded beginning of a file
    // Write it, initialize history with it
    fn init(&mut self) -> Result<()> {
        // TODO: Optimize to read in words of 32bits
        for _ in 0..self.current_window_size {
            match self.br.read_byte() {
                Ok(byte) => self.write_decoded(&Record::Literal { byte })?,
                Err(err) => {
                    return Err(Error::new(
                        err.kind(),
                        format!("Error while reading beginning of a file: {}", err),
                    ))
                }
            }
        }
        Ok(())
    }

    // Writes to file and updates history
    fn write_decoded(&mut self, record: &Record) -> Result<()> {
        let byte_vec: Vec<u8> = match record {
            Record::Literal { byte } => vec![*byte],
            Record::Reference { position, length } => {
                assert!(*length <= self.current_window_size);
                Vec::from(&self.history[*position..*position + *length])
            }
        };
        let bytes = byte_vec.as_slice();
        // Could hang?
        self.writer.write_all(bytes)?;

        // Update history
        let new_size = self.history.len() + bytes.len();
        if new_size > self.history_size {
            let to_pop = new_size - self.history_size;
            for _ in 0..to_pop {
                self.history.pop_front();
            }
        }

        for byte in bytes {
            self.history.push_back(*byte);
        }

        Ok(())
    }

    fn read_header(br: &mut BitReader<R, MSB>) -> Result<(usize, usize)> {
        let res: Result<(usize, usize)> = {
            let history_nbits = br.read_bits(BITS_FOR_HISTORY_ADDR_NBTIS)?;
            let match_len_nbits = br.read_bits(BITS_FOR_MATCH_LENGTH_NBITS)?;
            Ok((history_nbits as usize, match_len_nbits as usize))
        };
        match res {
            Ok(r) => Ok(r),
            Err(err) => Err(Error::new(
                err.kind(),
                format!("Error while reading header: {}", err),
            )),
        }
    }
}

pub fn decode<R: Read, W: Write>(reader: R, writer: W) -> Result<()> {
    let mut decoder = Decoder::new(reader, writer)?;
    decoder.decode()
}

// fn write_header<W: Write>(&self, bw: &mut BitWriter<W>) -> Result<()> {
//     bw.write_bits(self.history_addr_nbits as u32, BITS_FOR_HISTORY_ADDR_NBTIS)?;
//     bw.write_bits(self.match_length_nbits as u32, BITS_FOR_MATCH_LENGTH_NBITS)?;
//     Ok(())
// }
