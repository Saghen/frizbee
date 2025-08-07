use std::arch::x86_64::*;

/// Checks if the needle is wholly contained in the haystack, ignoring the exact order of the
/// bytes. For example, if the needle is "test", the haystack "tset" will return true. However,
/// the order does matter across 16 byte boundaries. The needle chars must include both the
/// uppercase and lowercase variants of the character.
///
/// Use a function with `#[target_feature(enable = "sse2,avx,avx2")]`
///
/// # Safety
/// When W > 16, the caller must ensure that the minimum length of the haystack is >= 16.
/// When W <= 16, the caller must ensure that the minimum length of the haystack is >= 8.
/// In all cases, the caller must ensure the needle.len() > 0 and that SSE2 and AVX2 are available.
#[inline(always)]
pub unsafe fn match_haystack_unordered_insensitive<const W: usize>(
    needle_simd: &[__m256i],
    haystack: &[u8; W],
) -> bool {
    let mut needle_iter = needle_simd.iter();
    let mut needle_char = *needle_iter.next().unwrap();

    // TODO: in theory, we could set the `start` to the last chunk from the previous run
    for start in (0..W).step_by(16) {
        let haystack_chunk = unsafe { _mm_loadu_epi8(haystack[start..].as_ptr() as *const i8) };
        let haystack_chunk = unsafe { _mm256_broadcastsi128_si256(haystack_chunk) };

        // For AVX2, we store the uppercase in the first 16 bytes, and the lowercase in the
        // last 16 bytes. This allows us to compare the uppercase and lowercase versions of
        // the needle char in the same comparison.
        loop {
            if unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi8(needle_char, haystack_chunk)) } == 0
            {
                // No match, advance to next chunk
                break;
            }

            // Progress to next needle char, if available
            if let Some(next_needle_char) = needle_iter.next() {
                needle_char = *next_needle_char;
            } else {
                return true;
            }
        }
    }

    false
}
