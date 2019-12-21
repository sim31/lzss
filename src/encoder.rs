//use bitbit::{BitReader, BitWriter, MSB};
use super::*;
use super::{history_reader::*, search};
use bitbit::BitWriter;
use std::cmp;
use std::io::{Read, Result, Write};

pub struct Encoder {
    threshold: u8,          // When to encode
    history_addr_nbits: u8, // Number of bits used for addressing history
    match_length_nbits: u8, // Number of bits used for specifying length of a match
    search_depth: u8, // 0 - longest match, 1 - first match, 2 - longest of the first two matches
}

impl Encoder {
    pub fn new(history_addr_nbits: u8, match_length_nbits: u8, search_depth: u8) -> Encoder {
        let history_interval_msg = format!("History address size must be in range of [{}, {}] bits", 
            MIN_HISTORY_ADDR_BITS, MAX_HISTORY_ADDR_BITS);
        let match_len_interval_msg = format!("Match length size must be in range of [{}, {}] bits", 
            MIN_MATCH_LENGTH_BITS, MAX_MATCH_LENGTH_BITS);
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

        let record_1_size: u8 = 1 + history_addr_nbits + match_length_nbits;
        let record_2_size: u8 = 1 + 8;
        let threshold: u8 = (record_1_size / record_2_size) + 1;

        Encoder {
            threshold,
            history_addr_nbits,
            match_length_nbits,
            search_depth,
        }
    }

    pub fn encode<R: Read, W: Write>(&self, reader: &mut R, writer: &mut W) -> Result<()> {
        let mut bw = BitWriter::new(&mut *writer);
        self.write_header(&mut bw)?;

        // Asserting that window size do not break the limits (that's why using those types for pow function)
        let history_size: HistoryAddress =
            HistoryAddress::pow(2 as HistoryAddress, self.history_addr_nbits as u32);
        let current_window_size: MatchLength =
            MatchLength::pow(2 as MatchLength, self.match_length_nbits as u32);
        // Increasing window size by threshold, because we won't be encoding matches shorter than threshold
        let current_window_size = (current_window_size as usize) + (self.threshold as usize);
        let history_size = history_size as usize;
        let (history_size, current_window_size) =
            (history_size as usize, current_window_size as usize);
        let mut reader = HistoryReader::new(reader, history_size, current_window_size)?;

        let (mut history, mut window) = reader.current();

        // println!("History: {:#x?}", history);
        // println!("Window: {:#x?}", window);
        let (threshold, search_depth) = (self.threshold as usize, self.search_depth as usize);
        let (mut match_pos, mut match_len): (usize, usize);
        while !window.is_empty() {
            let bmatch = search::best_match(history, window, threshold, search_depth);
            match_pos = bmatch.0;
            match_len = bmatch.1;
            let bytes_encoded = if match_len > 0 {
                assert!(match_len >= threshold);
                self.write_record_1(&mut bw, match_pos, match_len)?;
                match_len
            } else {
                self.write_record_2(&mut bw, window[0])?;
                1
            };
            assert!(match_len <= window.len());
            println!("pos: {}, len: {}", match_pos, match_len);

            let new = reader.next(bytes_encoded)?;
            // let new = reader.next(cmp::min(win_len, 5))?;
            history = new.0;
            window = new.1;
            assert!(history.len() <= history_size);
            // println!("History: {:#x?}", history);
            // println!("Window: {:#x?}", window);
            // println!("Run: {}", i);
        }

        bw.pad_to_byte()?;
        writer.flush()?;

        Ok(())
    }

    fn write_record_1<W: Write>(&self, bw: &mut BitWriter<W>, pos: usize, length: usize) -> Result<()> {
        bw.write_bit(false)?;
        // Downcasting. But we limit possible positions (and lengths) in the beginning (when creating Decoder).
        bw.write_bits(pos as u32, self.history_addr_nbits as usize)?;
        // Not encoding with this type of record if it's shorter match than threshold
        let enc_len = length + (self.threshold as usize);
        bw.write_bits(enc_len as u32, self.history_addr_nbits as usize)?;
        Ok(())
    }

    fn write_record_2<W: Write>(&self, bw: &mut BitWriter<W>, byte: u8) -> Result<()> {
        bw.write_bit(true)?;
        bw.write_byte(byte)?;
        Ok(())
    }


    fn write_header<W: Write>(&self, bw: &mut BitWriter<W>) -> Result<()> {
        bw.write_bits(self.history_addr_nbits as u32, BITS_FOR_HISTORY_ADDR_NBTIS)?;
        bw.write_bits(self.match_length_nbits as u32, BITS_FOR_MATCH_LENGTH_NBITS)?;
        Ok(())
    }
}
