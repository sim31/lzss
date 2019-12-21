use std::cmp;

// Find first byte string in sequence matching subsequence or beginning of subsequence
// Returns the position of the first match and length of a matching byte string.
pub fn first_match(sequence: &[u8], subsequence: &[u8]) -> (usize, usize) {
    let (subs_len, seq_len) = (subsequence.len(), sequence.len());
    assert!(seq_len >= subs_len);

    let mut match_count = 0;
    let mut match_start = 0;
    for (pos, item) in sequence.iter().enumerate() {
        if *item == subsequence[match_count] {
            if match_count == 0 {
                // First match
                match_start = pos;
            }
            match_count += 1;

            if match_count == subs_len {
                // Whole match was found
                return (match_start, match_count);
            }
        } else if match_count > 0 {
            // Matching string of bytes has ended
            return (match_start, match_count);
        }
    }

    (match_start, match_count)
}

// Find best match, based on search_depth and threshold
// threshold: minimum length of a match
// search_depth:  0 - longest match, 1 - first match, 2 - longest of the first two matches...
// Returns it's position and length in sequence
pub fn best_match(
    sequence: &[u8],
    subsequence: &[u8],
    threshold: usize,
    search_depth: usize,
) -> (usize, usize) {
    // println!("\n----\nsequence: \"{}\"", std::str::from_utf8(sequence).unwrap());
    // println!("subsequence: \"{}\"", std::str::from_utf8(subsequence).unwrap());
    // println!("threshold: {}", threshold);
    let mut best_match: (usize, usize) = (0, 0);
    let (mut matches_found, mut pos): (usize, usize) = (0, 0);
    let seq_len = sequence.len();

    while pos < seq_len && (search_depth == 0 || matches_found < search_depth) {
        let seq = &sequence[pos..seq_len];
        let subs_len = cmp::min(subsequence.len(), seq.len());
        let subs = &subsequence[0..subs_len];
        let nmatch = first_match(seq, subs);
        let match_pos = nmatch.0 + pos; // match_pos we get back is relative to the beginning of the search
        let match_len = nmatch.1;
        if match_len > 0 {
            pos = match_pos + 1; // Continue search from the next byte
            if match_len >= threshold {
                if match_len > best_match.1 {
                    best_match = (match_pos, match_len);
                }
                matches_found += 1; // Only counting matches which reach threshold
                println!(
                    "new_match: ({}, {}), matches_found: {}, best_match: ({}, {}), new pos: {}",
                    match_pos, match_len, matches_found, best_match.0, best_match.1, pos
                );
            }
        } else {
            // No match was found. Means we have searched all of it.
            break;
        }
    }
    best_match
}

#[cfg(test)]
mod test {
    use super::*;
    use rand;

    fn fill_with_random(buffer: &mut [u8], prob: u8) {
        for el in buffer {
            *el = rand::random::<u8>() % prob;
        }
    }

    #[test]
    fn are_matches_valid() {
        for _ in 0..12 {
            let mut sequence: [u8; 16] = [0; 16];
            fill_with_random(&mut sequence, 4);
            let subsequence: [u8; 8] = [0, 3, 3, 2, 1, 0, 3, 0];

            let (pos, len) = first_match(&sequence, &subsequence);
            // println!("sequence: {:#x?}", &sequence[0..sequence.len()]);
            // println!("subsequence: {:#x?}", &subsequence[0..subsequence.len()]);
            println!("Match: {}, {}", pos, len);
            assert!(
                sequence[pos..pos + len] == subsequence[0..len],
                "This is not a match"
            );
        }
    }

    #[test]
    fn alternative_best_match_impl_test() {
        for _ in 0..8 {
            let mut sequence: [u8; 64] = [0; 64];
            fill_with_random(&mut sequence, 4);
            let subsequence: [u8; 8] = [0, 3, 1, 2, 1, 0, 3, 0];

            let threshold = 3;
            let (pos, len) = best_match(&sequence, &subsequence, threshold, 0);

            let alt_find = || {
                for len in (threshold..subsequence.len()).rev() {
                    if let Some(pos) = sequence
                        .windows(len)
                        .position(|window| window == &subsequence[0..len])
                    {
                        return (pos, len);
                    }
                }
                (0, 0)
            };

            let (pos1, len1) = alt_find();
            assert!(pos == pos1 && len == len1, "Results don't match");
            println!("pos: {}, len: {}", pos, len);
        }
    }
}
