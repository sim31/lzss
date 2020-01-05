//use bitbit::{BitReader, BitWriter, MSB};
use super::*;
use super::{history_reader::*, search};
use bitbit::BitWriter;
use log::debug;
use std::io::{Read, Result, Write};

pub struct Encoder {
    threshold: u8,          // When to encode
    history_addr_nbits: u8, // Number of bits used for addressing history
    match_length_nbits: u8, // Number of bits used for specifying length of a match
    search_depth: u8, // 0 - longest match, 1 - first match, 2 - longest of the first two matches
    bits_written: usize,
}

impl Encoder {
    pub fn new(history_addr_nbits: u8, match_length_nbits: u8, search_depth: u8) -> Encoder {
        let history_interval_msg = format!(
            "History address size must be in range of [{}, {}] bits",
            MIN_HISTORY_ADDR_BITS, MAX_HISTORY_ADDR_BITS
        );
        let match_len_interval_msg = format!(
            "Match length size must be in range of [{}, {}] bits",
            MIN_MATCH_LENGTH_BITS, MAX_MATCH_LENGTH_BITS
        );
        assert!(
            history_addr_nbits >= 3 && history_addr_nbits <= 31,
            history_interval_msg
        );
        assert!(
            match_length_nbits >= 2 && match_length_nbits <= 15,
            match_len_interval_msg
        );
        assert!(
            history_addr_nbits > match_length_nbits,
            "History size has to be bigger than current window size"
        );

        let threshold =
            calc_threshold(history_addr_nbits as usize, match_length_nbits as usize) as u8;

        Encoder {
            threshold,
            history_addr_nbits,
            match_length_nbits,
            search_depth,
            bits_written: 0,
        }
    }

    pub fn encode<R: Read, W: Write>(&mut self, reader: &mut R, writer: &mut W) -> Result<()> {
        let mut bw = BitWriter::new(&mut *writer);
        self.write_header(&mut bw)?;

        // Asserting that window size do not break the limits (that's why using those types for pow function)
        let history_size: HistoryAddress =
            HistoryAddress::pow(2 as HistoryAddress, self.history_addr_nbits as u32);
        let current_window_size: MatchLength =
            MatchLength::pow(2 as MatchLength, self.match_length_nbits as u32);
        // Increasing window size by threshold, because we won't be encoding matches shorter than threshold
        // Decreasing window size by one because we cannot encode the largest possible length of 2^n with n bits.
        let current_window_size = (current_window_size as usize) + (self.threshold as usize) - 1;
        let history_size = history_size as usize;
        let (history_size, current_window_size) =
            (history_size as usize, current_window_size as usize);
        let mut reader = HistoryReader::new(reader, history_size, current_window_size)?;

        let (mut history, mut window) = reader.current();
        self.write_initial_history(&mut bw, Vec::from(history).as_slice())?;

        // debug!("History: {:#x?}", history);
        // debug!("Window: {:#x?}", window);
        let (threshold, search_depth) = (self.threshold as usize, self.search_depth as usize);
        let (mut match_pos, mut match_len): (usize, usize);
        while !window.is_empty() {
            let bmatch = search::best_match(history, window, threshold, search_depth);
            match_pos = bmatch.0;
            match_len = bmatch.1;
            let bytes_encoded = if match_len > 0 {
                assert!(match_len >= threshold);
                self.write_reference_record(&mut bw, match_pos, match_len)?;
                match_len
            } else {
                self.write_literal_record(&mut bw, window[0])?;
                1
            };
            assert!(match_len <= window.len());
            assert!(match_pos < history.len());
            // debug!("pos: {}, len: {}", match_pos, match_len);

            let new = reader.next(bytes_encoded)?;
            // let new = reader.next(cmp::min(win_len, 5))?;
            history = new.0;
            window = new.1;
            assert!(history.len() <= history_size);
            debug!("History: {:#x?}", history);
            debug!("Window: {:#x?}", window);
            // debug!("Run: {}", i);
        }

        self.write_ending(&mut bw)?;

        writer.flush()?;

        Ok(())
    }

    fn write_reference_record<W: Write>(
        &mut self,
        bw: &mut BitWriter<W>,
        pos: usize,
        length: usize,
    ) -> Result<()> {
        bw.write_bit(RECORD_TYPE_REFERENCE)?;
        // FIXME: Store nbits fields as usize
        let history_addr_nbits = self.history_addr_nbits as usize;
        let match_length_nbits = self.match_length_nbits as usize;
        // Downcasting. But we limit possible positions (and lengths) in the beginning (when creating Decoder).
        bw.write_bits(pos as u32, history_addr_nbits)?;
        // Not encoding with this type of record if it's shorter match than threshold
        let enc_len = (length as u32) - (self.threshold as u32);
        bw.write_bits(enc_len, match_length_nbits)?;

        self.bits_written += 1 + history_addr_nbits + match_length_nbits;

        debug!(
            "Record: Reference {{ position: {}, length: {} }}",
            pos, length
        );

        Ok(())
    }

    fn write_literal_record<W: Write>(&mut self, bw: &mut BitWriter<W>, byte: u8) -> Result<()> {
        bw.write_bit(RECORD_TYPE_LITERAL)?;
        bw.write_byte(byte)?;

        self.bits_written += 1 + 8;

        debug!("Record: Literal {{ byte: {} }}", byte);
        Ok(())
    }

    fn write_ending<W: Write>(&mut self, bw: &mut BitWriter<W>) -> Result<()> {
        // There are two valid ways for an archive file to end:
        //  1. At the byte boundary (if the end of the last record is at the byte boundary)
        //  2. Or if last record does not end at byte boundary,
        //  it has to end with a 1 and padding till the next byte boundary
        //  (which creates an invalid literal record - that's how we know it's the end).
        // If we get EOF when reading type bit, it's the first type of ending.
        // If we get a literal type bit and EOF while reading it's byte, it means it's the second type of ending.
        // Every other case of EOF is interpreted as InvalidData error.
        if self.bits_written % 8 != 0 {
            bw.write_bit(RECORD_TYPE_LITERAL)?;
            self.bits_written += 1;
        }
        bw.pad_to_byte()?;
        debug!("Bits written: {}", self.bits_written);
        Ok(())
    }

    // Writes initial history un-encoded.
    // File needs to begin this way so that decoder has some dictionary to start with
    fn write_initial_history<W: Write>(
        &mut self,
        bw: &mut BitWriter<W>,
        bytes: &[u8],
    ) -> Result<()> {
        // TODO: Optimize to write in words of 32 bits (as BitWriter allows it)
        // But then you have endiandness to worry about
        for byte in bytes {
            bw.write_byte(*byte)?;
        }

        self.bits_written += bytes.len() * 8;

        // debug!("Initial history: {}", std::str::from_utf8_unchecked(bytes));
        debug!("Initial history: {:?}", bytes);
        debug!("Initial history length: {}", bytes.len());
        Ok(())
    }

    fn write_header<W: Write>(&mut self, bw: &mut BitWriter<W>) -> Result<()> {
        bw.write_bits(self.history_addr_nbits as u32, BITS_FOR_HISTORY_ADDR_NBTIS)?;
        bw.write_bits(self.match_length_nbits as u32, BITS_FOR_MATCH_LENGTH_NBITS)?;
        debug!(
            "Header: ({}, {})",
            self.history_addr_nbits, self.match_length_nbits
        );
        self.bits_written += BITS_FOR_HISTORY_ADDR_NBTIS + BITS_FOR_MATCH_LENGTH_NBITS;
        Ok(())
    }
}
