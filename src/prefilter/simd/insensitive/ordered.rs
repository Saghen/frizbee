use super::super::overlapping_load;
use std::simd::{Simd, cmp::SimdPartialEq};

#[inline(always)]
pub fn match_haystack_insensitive(needle: &[(u8, u8)], haystack: &[u8]) -> bool {
    let len = haystack.len();

    let mut needle_iter = needle
        .iter()
        .map(|&(c1, c2)| (Simd::splat(c1), Simd::splat(c2)));
    let mut needle_char = needle_iter.next().unwrap();

    for start in (0..len).step_by(16) {
        let haystack_chunk = overlapping_load(haystack, start, len);

        let mut last_match_idx = None;
        loop {
            let mut mask = haystack_chunk.simd_eq(needle_char.0).to_bitmask()
                | haystack_chunk.simd_eq(needle_char.1).to_bitmask();

            // If we've already found a match on this chunk, 0 out the bits that come before the
            // last match
            if let Some(last_match_idx) = last_match_idx {
                mask &= u64::MAX << (last_match_idx + 1);
            }

            if mask != 0 {
                // Progress to next needle char, if available
                if let Some(next_needle_char) = needle_iter.next() {
                    needle_char = next_needle_char;
                } else {
                    return true;
                }

                // Get the number of leading zeros
                // Note that the mask is flipped from the comparison:
                // let haystack = _mm_setr_epi8(0,0,0,42,0,0,0,0,0,0,0,0,0,0,0,0);
                // let needle = _mm_set1_epi8(42);
                // Mask is 0000000000001000
                let idx = mask.trailing_zeros() as usize;

                // Reached end of haystack, advance to next chunk
                if idx == 15 {
                    break;
                }
                last_match_idx = Some(idx as usize);
            } else {
                // Advance to next chunk
                break;
            }
        }
    }

    false
}
