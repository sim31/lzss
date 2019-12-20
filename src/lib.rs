mod history_reader;

use bitbit::{BitReader, BitWriter, MSB};
use std::io::{BufReader, BufWriter, Read, Write, Result};
use std::collections::VecDeque;
use history_reader::*;

type HistoryAddress = u32;
type MatchLength = u16;

// Number of bits needed to encode history address size (in bits) in the header
const BITS_FOR_HISTORY_ADDR_NBTIS: usize = 5; 
// Number of bits needed to encode max match length size (in bits) in the header
const BITS_FOR_MATCH_LENGTH_NBITS: usize = 4;

pub struct Encoder {
    threshold: u8,          // When to encode
    history_addr_nbits: u8, // Number of bits used for addressing history
    match_length_nbits: u8, // Number of bits used for specifying length of a match
    search_depth: u8,       // 0 - longest match, 1 - first match, 2 - longest of the first two matches
}

impl Encoder {
    pub fn new(
        history_addr_nbits: u8,
        match_length_nbits: u8,
        search_depth: u8,
    ) -> Encoder {
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

        let history_size: HistoryAddress =
            HistoryAddress::pow(2 as HistoryAddress, self.history_addr_nbits as u32);
        let current_window_size: MatchLength =
            MatchLength::pow(2 as MatchLength, self.match_length_nbits as u32);
        let mut reader = HistoryReader::new(reader, history_size as usize, current_window_size as usize)?;

        let (mut window, mut history) = reader.current();

        println!("History: {:#x?}", history);
        println!("Window: {:#x?}", window);
        while !window.is_empty() {
            // FIXME: Convert to usize once
            let win_len = window.len();
            let new = reader.next(win_len)?;
            history = new.0;
            window = new.1;

            println!("History: {:#x?}", history);
            println!("Window: {:#x?}", window);
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
