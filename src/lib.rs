use bitbit::{BitReader, BitWriter, MSB};
use std::io::{BufReader, BufWriter, Read, Write, Result};

type HistoryAddress = u32;
type MatchLength = u16;

// Number of bits needed to encode history address size (in bits) in the header
const BITS_FOR_HISTORY_ADDR_NBTIS: usize = 5; 
// Number of bits needed to encode max match length size (in bits) in the header
const BITS_FOR_MATCH_LENGTH_NBITS: usize = 4;

pub struct Encoder {
    history: Vec<u8>,
    current_window: Vec<u8>,
    threshold: u8,    // When to encode
    history_addr_nbits: u8, // Number of bits used for addressing history
    match_length_nbits: u8, // Number of bits used for specifying length of a match
    start_at: u8,     // At what byte of the input file to start encoding
    search_depth: u8, // 0 - longest match, 1 - first match, 2 - longest of the first two matches
}

impl Encoder {
    pub fn new(
        history_addr_nbits: u8,
        match_length_nbits: u8,
        search_depth: u8,
    ) -> Encoder {
        assert!(
            history_addr_nbits >= 4,
            "History address size must be in range of [4, 31] bits"
        );
        assert!(
            match_length_nbits >= 2,
            "Current window address size must be in range of [2, 15] bits"
        );
        let history_size: HistoryAddress =
            HistoryAddress::checked_pow(2 as HistoryAddress, history_addr_nbits as u32)
                .expect("History address size must be in range of [4, 31] bits");
        let now_size: MatchLength =
            MatchLength::checked_pow(2 as MatchLength, match_length_nbits as u32)
                .expect("Current window address size must be in range of [2, 15] bits");

        let record_1_size: u8 = 1 + history_addr_nbits + match_length_nbits;
        let record_2_size: u8 = 1 + 8;
        let threshold: u8 = (record_1_size / record_2_size) + 1;
        let start_at: u8 = (record_1_size / 8) + 1; // That's when it becomes to be possible to get compression

        Encoder {
            history: Vec::with_capacity(history_size as usize),
            current_window: Vec::with_capacity(now_size as usize),
            threshold,
            history_addr_nbits,
            match_length_nbits,
            start_at,
            search_depth,
        }
    }

    pub fn encode<R: Read, W: Write>(&self, reader: &mut R, writer: &mut W) -> Result<()> {
        let mut br: BitReader<_, MSB> = BitReader::new(reader);
        let num = br.read_bits(5).unwrap();
        println!("{}", num);

        let mut bw = BitWriter::new(&mut *writer);

        self.write_header(&mut bw)?;

        bw.pad_to_byte()?;
        writer.flush()?;

        Ok(())
    }

    fn write_header<W: Write>(&self, bw: &mut BitWriter<W>) -> Result<()> {
        bw.write_bits(self.history_addr_nbits as u32, BITS_FOR_HISTORY_ADDR_NBTIS)?;
        bw.write_bits(self.match_length_nbits as u32, BITS_FOR_MATCH_LENGTH_NBITS)?;
        Ok(())
    }
}
