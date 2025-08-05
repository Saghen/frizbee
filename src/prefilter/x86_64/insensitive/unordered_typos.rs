use super::super::overlapping_load;
use std::arch::x86_64::*;

/// Checks if the needle is wholly contained in the haystack, ignoring the exact order of the
/// bytes. For example, if the needle is "test", the haystack "tset" will return true. However,
/// the order does matter across 16 byte boundaries. The needle chars must include both the
/// uppercase and lowercase variants of the character.
///
/// Fastest with SSE2, AVX, and AVX2, but still very fast with just SSE2. Use a function with
/// `#[target_feature(enable = "sse2,avx,avx2")]` or `#[target_feature(enable = "sse2")]`
///
/// # Safety
/// When W > 16, the caller must ensure that the minimum length of the haystack is >= 16.
/// When W <= 16, the caller must ensure that the minimum length of the haystack is >= 8.
/// In all cases, the caller must ensure the needle.len() > 0 and that SSE2 is available.
#[inline(always)]
pub unsafe fn match_haystack_unordered_typos_insensitive<const W: usize>(
    needle: &[(u8, u8)],
    haystack: &[u8],
    max_typos: u16,
) -> bool {
    unsafe {
        let len = haystack.len();

        let mut needle_iter = needle.iter().map(|&(c1, c2)| {
            let c1 = _mm_set1_epi8(c1 as i8);
            let c2 = _mm_set1_epi8(c2 as i8);
            _mm_unpacklo_epi64(c1, c2)
        });
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
                    // Check if any of the chars in the needle_char are in the haystack_chunk
                    let cmp = _mm_cmpistri::<_SIDD_CMP_EQUAL_ANY>(needle_char, haystack_chunk);
                    // No match, advance to next chunk
                    if cmp == 16 {
                        break;
                    }

                    // Progress to next needle char, if available
                    if let Some(next_needle_char) = needle_iter.next() {
                        needle_char = next_needle_char;
                    } else {
                        return true;
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
}
