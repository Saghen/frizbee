use super::super::overlapping_load;
use std::simd::{Simd, cmp::SimdPartialEq};

#[inline(always)]
pub fn match_haystack_unordered(needle: &[u8], haystack: &[u8]) -> bool {
    let len = haystack.len();

    let mut needle_iter = needle.iter().map(|&c| Simd::<u8, 16>::splat(c));
    let mut needle_char = needle_iter.next().unwrap();

    for start in (0..len).step_by(16) {
        let haystack_chunk = overlapping_load(haystack, start, len);

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

    false
}
