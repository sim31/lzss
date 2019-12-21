type HistoryAddress = u32;
type MatchLength = u16;

// Number of bits needed to encode history address size (in bits) in the header
pub const BITS_FOR_HISTORY_ADDR_NBTIS: usize = 5;
// Number of bits needed to encode max match length size (in bits) in the header
pub const BITS_FOR_MATCH_LENGTH_NBITS: usize = 4;

mod history_reader;
pub mod encoder;
pub mod search;


