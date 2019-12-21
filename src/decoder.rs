use super::*;
use bitbit::{BitReader, MSB};
use std::io::{Read, Result, Write};
use slice_deque::SliceDeque;

// pub fn decode<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<()> {
//     let mut br: BitReader<_, MSB> = BitReader::new(reader);

//     let (history_addr_nbits, match_length_nbits) = read_header(&mut br)?;
//     assert!(history_addr_nbits >= MIN_HISTORY_ADDR_BITS && history_addr_nbits <= MAX_HISTORY_ADDR_BITS);
//     assert!(match_length_nbits >= MIN_MATCH_LENGTH_BITS && match_length_nbits <= MAX_MATCH_LENGTH_BITS);

//     let history_size: usize = usize::pow(2, history_addr_nbits as u32);
//     let history = SliceDeque::with_capacity(history_size);

// }

fn read_header<R: Read>(br: &mut BitReader<R, MSB>) -> Result<(usize, usize)> {
    let history_nbits = br.read_bits(BITS_FOR_HISTORY_ADDR_NBTIS)?;
    let match_len_nbits = br.read_bits(BITS_FOR_MATCH_LENGTH_NBITS)?;
    Ok((history_nbits as usize, match_len_nbits as usize))
}

// fn write_header<W: Write>(&self, bw: &mut BitWriter<W>) -> Result<()> {
//     bw.write_bits(self.history_addr_nbits as u32, BITS_FOR_HISTORY_ADDR_NBTIS)?;
//     bw.write_bits(self.match_length_nbits as u32, BITS_FOR_MATCH_LENGTH_NBITS)?;
//     Ok(())
// }