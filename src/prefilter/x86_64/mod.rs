use std::arch::x86_64::*;

mod insensitive;
mod sensitive;

pub use insensitive::*;
pub use sensitive::*;

/// Loads the cased needle into a __m256i vector, where the first 16 bytes are the uppercase
/// and the last 16 bytes are the lowercase version of the needle.
///
/// # Safety
/// Caller must ensure that SSE2, AVX, and AVX2 are available at runtime
#[target_feature(enable = "sse2,avx,avx2")]
pub unsafe fn needle_to_avx2(needle_cased: &[(u8, u8)]) -> Vec<std::arch::x86_64::__m256i> {
    needle_cased
        .iter()
        .map(|&(c1, c2)| unsafe {
            _mm256_loadu2_m128i(&_mm_set1_epi8(c1 as i8), &_mm_set1_epi8(c2 as i8))
        })
        .collect::<Vec<_>>()
}

/// Loads a chunk of 16 bytes from the haystack, with overlap when remaining bytes < 16,
/// since it's dramatically faster than a memcpy.
///
/// If the haystack the number of remaining bytes is < 16, and the total length is > 16,
/// the last 16 bytes are loaded from the end of the haystack.
///
/// If the haystack is < 16 bytes, we load the first 8 bytes from the haystack, and the last 8
/// bytes, and combine them into a single vector.
///
/// # Safety
/// Caller must ensure that haystack length >= 8 when W <= 16
/// Caller must ensure that haystack length >= 16 when W > 16
#[inline(always)]
pub unsafe fn overlapping_load(haystack: &[u8], start: usize, len: usize) -> __m128i {
    unsafe {
        match len {
            0..=7 => unreachable!(),
            8 => _mm_loadl_epi64(haystack.as_ptr() as *const __m128i),
            // Loads 8 bytes from the start of the haystack, and 8 bytes from the end of the haystack
            // and combines them into a single vector. Much faster than a memcpy
            9..=15 => {
                let low = _mm_loadl_epi64(haystack.as_ptr() as *const __m128i);
                let high_start = len - 8;
                let high = _mm_loadl_epi64(haystack[high_start..].as_ptr() as *const __m128i);
                _mm_unpacklo_epi64(low, high)
            }
            16 => _mm_loadu_si128(haystack.as_ptr() as *const __m128i),
            // Avoid reading past the end, instead re-read the last 16 bytes
            _ => _mm_loadu_si128(haystack[start.min(len - 16)..].as_ptr() as *const __m128i),
        }
    }
}
