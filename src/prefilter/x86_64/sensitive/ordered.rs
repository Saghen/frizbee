use super::super::overlapping_load;
use std::arch::x86_64::*;

/// Checks if the needle is wholly contained in the haystack, allowing for gaps between needle
/// bytes in the haystack. For example "test" on "te__st" will return true.
///
/// Fastest with SSE2, AVX, and AVX2, but still very fast with just SSE2. Use a function with
/// `#[target_feature(enable = "sse2,avx,avx2")]` or `#[target_feature(enable = "sse2")]`
///
/// # Safety
/// When W > 16, the caller must ensure that the minimum length of the haystack is >= 16.
/// When W <= 16, the caller must ensure that the minimum length of the haystack is >= 8.
/// In all cases, the caller must ensure the needle.len() > 0 and that SSE2 is available.
#[inline(always)]
pub unsafe fn match_haystack(needle: &[u8], haystack: &[u8]) -> bool {
    let len = haystack.len();

    let mut needle_iter = needle.iter().map(|&c| unsafe { _mm_set1_epi8(c as i8) });
    let mut needle_char = needle_iter.next().unwrap();

    for start in (0..len).step_by(16) {
        if start >= len {
            return false;
        }

        let haystack_chunk = unsafe { overlapping_load(haystack, start, len) };

        let mut last_match_idx = None;
        loop {
            // Compare each byte (0xFF if equal, 0x00 if not)
            let cmp = unsafe { _mm_cmpeq_epi8(needle_char, haystack_chunk) };

            // Convert comparison result to bitmask
            let mut mask = unsafe { _mm_movemask_epi8(cmp) } as u16;

            // If we've already found a match on this chunk, 0 out the bits that come before the
            // last match
            if let Some(last_match_idx) = last_match_idx {
                mask &= u16::MAX << (last_match_idx + 1);
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
                let idx = unsafe { _tzcnt_u16(mask) } as usize;

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
