type HistoryAddress = u32;
type MatchLength = u16;

// Number of bits needed to encode history address size (in bits) in the header
pub const BITS_FOR_HISTORY_ADDR_NBTIS: usize = 5;
// Number of bits needed to encode max match length size (in bits) in the header
pub const BITS_FOR_MATCH_LENGTH_NBITS: usize = 4;
pub const MAX_HISTORY_ADDR_BITS: usize = 31;
pub const MIN_HISTORY_ADDR_BITS: usize = 3;
pub const MAX_MATCH_LENGTH_BITS: usize = 15;
pub const MIN_MATCH_LENGTH_BITS: usize = 2;
pub const RECORD_TYPE_REFERENCE: bool = false;
pub const RECORD_TYPE_LITERAL: bool = true;

pub mod decoder;
pub mod encoder;
mod history_reader;
pub mod search;

fn calc_threshold(history_addr_nbits: usize, match_len_nbits: usize) -> usize {
    let record_1_size = 1 + history_addr_nbits + match_len_nbits;
    let record_2_size = 1 + 8;
    (record_1_size / record_2_size) + 1
}
