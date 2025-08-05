use super::super::overlapping_load;
use std::simd::{cmp::SimdPartialEq, Simd};

#[inline(always)]
pub fn match_haystack_unordered_insensitive<const W: usize>(
    needle: &[(u8, u8)],
    haystack: &[u8],
) -> bool {
    let len = haystack.len();

    let mut needle_iter = needle
        .iter()
        .map(|&(c1, c2)| (Simd::splat(c1), Simd::splat(c2)));
    let mut needle_char = needle_iter.next().unwrap();

    for start in (0..W).step_by(16) {
        let haystack_chunk = overlapping_load::<W>(haystack, start, len);

        loop {
            if haystack_chunk.simd_eq(needle_char.0).any()
                || haystack_chunk.simd_eq(needle_char.1).any()
            {
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
