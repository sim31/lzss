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
        assert!(
            history_addr_nbits >= 3 && history_addr_nbits <= 31,
            "History address size must be in range of [3, 31] bits"
        );
        assert!(
            match_length_nbits >= 2 && match_length_nbits <= 15,
            "Current window address size must be in range of [2, 15] bits"
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
            assert!(match_len <= window.len());
            println!("pos: {}, len: {}", match_pos, match_len);

            let move_bytes = cmp::max(1, match_len);
            let new = reader.next(move_bytes)?;
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

    // pub fn decode<R: Read, W: Write>(&self, reader: &mut R, writer: &mut W) -> Result<()> {
    //     let mut br: BitReader<_, MSB> = BitReader::new(reader);
    //     let num = br.read_bits(5).unwrap();
    //     println!("{}", num);
    // }

    fn write_header<W: Write>(&self, bw: &mut BitWriter<W>) -> Result<()> {
        bw.write_bits(self.history_addr_nbits as u32, BITS_FOR_HISTORY_ADDR_NBTIS)?;
        bw.write_bits(self.match_length_nbits as u32, BITS_FOR_MATCH_LENGTH_NBITS)?;
        Ok(())
    }
}
