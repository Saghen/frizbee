use super::super::overlapping_load;
use std::simd::{cmp::SimdPartialEq, Simd};

#[inline(always)]
pub fn match_haystack_unordered_typos<const W: usize>(
    needle: &[u8],
    haystack: &[u8],
    max_typos: u16,
) -> bool {
    let len = haystack.len();

    let mut needle_iter = needle.iter().map(|&c| Simd::<u8, 16>::splat(c));
    let mut needle_char = needle_iter.next().unwrap();

    let mut typos = 0;
    loop {
        if typos > max_typos as usize {
            return false;
        }

        // TODO: this is slightly incorrect, because if we match on the third chunk,
        // we would only scan from the third chunk onwards for the next needle. Technically,
        // we should scan from the beginning of the haystack instead, but I believe the
        // previous memchr implementation had the same bug.
        for start in (0..W).step_by(16) {
            let haystack_chunk = overlapping_load::<W>(haystack, start, len);

            loop {
                if haystack_chunk.simd_eq(needle_char).any() {
                    // Progress to next needle char, if available
                    if let Some(next_needle_char) = needle_iter.next() {
                        needle_char = next_needle_char;
                    } else {
                        return true;
                    }
                } else {
                    // Advance to next chunk
                    break;
                }
            }
        }

        typos += 1;
        if typos > max_typos as usize {
            return false;
        }

        if let Some(next_needle_char) = needle_iter.next() {
            needle_char = next_needle_char;
        } else {
            return true;
        }
    }
}
