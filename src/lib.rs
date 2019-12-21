type HistoryAddress = u32;
type MatchLength = u16;

// Number of bits needed to encode history address size (in bits) in the header
pub const BITS_FOR_HISTORY_ADDR_NBTIS: usize = 5;
// Number of bits needed to encode max match length size (in bits) in the header
pub const BITS_FOR_MATCH_LENGTH_NBITS: usize = 4;
pub const MAX_HISTORY_ADDR_BITS: usize  = 31;
pub const MIN_HISTORY_ADDR_BITS: usize  = 3;
pub const MAX_MATCH_LENGTH_BITS: usize  = 15;
pub const MIN_MATCH_LENGTH_BITS: usize  = 2;

pub mod encoder;
pub mod decoder;
pub mod search;
mod history_reader;
