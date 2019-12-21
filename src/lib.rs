mod history_reader;

//use bitbit::{BitReader, BitWriter, MSB};
use bitbit::BitWriter;
use history_reader::*;
use std::io::{Read, Result, Write};
use std::cmp;

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

    // Returns the position of the first match and length of a matching byte string. 
    // Match might be smaller than subsequence.
    pub fn find_first_match(&self, sequence: &[u8], subsequence: &[u8]) -> (usize, usize) {
        let (subs_len, seq_len) = (subsequence.len(), sequence.len());
        assert!(seq_len >= subs_len);

        let mut match_count = 0;
        let mut match_start = 0;
        for (pos, item) in sequence.iter().enumerate() {
            if *item == subsequence[match_count] {
                if match_count == 0 { // First match
                    match_start = pos;
                }
                match_count += 1;
                
                if match_count == subs_len { // Whole match was found
                    return (match_start, match_count);
                }
            } else if match_count > 0 { // Matching string of bytes has ended
                return (match_start, match_count);
            }
        }

        (match_start, match_count)
    }

    // Find best match, based on search_depth
    // Returns it's position and length in sequence
    pub fn find_best_match(&self, sequence: &[u8], subsequence: &[u8]) -> (usize, usize) {
        let mut best_match: (usize, usize) = (0, 0);
        let (mut matches_found, mut pos) = (0, 0);
        let seq_len = sequence.len();

        while pos < seq_len && (self.search_depth == 0 || matches_found < self.search_depth) {
            let seq = &sequence[pos..seq_len];
            let subs_len = cmp::min(subsequence.len(), seq.len());
            let subs = &subsequence[0..subs_len];
            let (match_pos, match_len) = self.find_first_match(seq, subs);
            if match_len > 0 {
                pos = match_pos + 1;  // Continue search from the next byte
                if match_len >= self.threshold as usize {
                    if match_len > best_match.1 {
                        best_match = (match_pos, match_len);
                    }            
                    matches_found += 1; // Only counting matches which reach threshold
                }
            } else { // No match was found. Means we have searched all of it.
                break;
            }
        }
        best_match
    }

    pub fn encode<R: Read, W: Write>(&self, reader: &mut R, writer: &mut W) -> Result<()> {
        let mut bw = BitWriter::new(&mut *writer);
        self.write_header(&mut bw)?;

        let history_size: HistoryAddress =
            HistoryAddress::pow(2 as HistoryAddress, self.history_addr_nbits as u32);
        let current_window_size: MatchLength =
            MatchLength::pow(2 as MatchLength, self.match_length_nbits as u32);
        let mut reader =
            HistoryReader::new(reader, history_size as usize, current_window_size as usize)?;

        let (mut history, mut window) = reader.current();

        println!("History: {:#x?}", history);
        println!("Window: {:#x?}", window);
        let (mut match_pos, mut match_len) = (0, 0);
        while !window.is_empty() {
            let win_len = window.len();
            let move_bytes = cmp::min(win_len, cmp::min(1, match_len));
            //let new = reader.next(move_bytes)?;
            let new = reader.next(cmp::min(win_len, 5))?;
            history = new.0;
            window = new.1;
            println!("History_size: {}, {}", history.len(), history_size);
            assert!(history.len() <= history_size as usize);

            // let bmatch = self.find_best_match(history, window);
            // match_pos = bmatch.0;
            // match_len = bmatch.1;
            // println!("pos: {}, len: {}", match_pos, match_len);

            println!("History: {:#x?}", history);
            println!("Window: {:#x?}", window);
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
