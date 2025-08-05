use std::simd::Simd;

mod insensitive;
mod sensitive;

pub use insensitive::*;
pub use sensitive::*;

/// Loads a chunk of 16 bytes from the haystack, with overlap when remaining bytes < 16,
/// since it's dramatically faster than a memcpy.
///
/// If the haystack the number of remaining bytes is < 16, and the total length is > 16,
/// the last 16 bytes are loaded from the end of the haystack.
///
/// If the haystack is < 16 bytes, we load the first 8 bytes from the haystack, and the last 8
/// bytes, and combine them into a single vector.
#[inline(always)]
pub fn overlapping_load<const W: usize>(haystack: &[u8], start: usize, len: usize) -> Simd<u8, 16> {
    if W <= 16 {
        match len {
            0..8 => unreachable!(),
            8 => Simd::load_or_default(&haystack[0..8]),
            // Loads 8 bytes from the start of the haystack, and 8 bytes from the end of the haystack
            // and combines them into a single vector. Much faster than a memcpy
            9..=15 => Simd::load_or_default(&haystack[0..len]),
            _ => Simd::from_slice(haystack),
        }
    } else {
        // Avoid reading past the end, instead re-read the last 16 bytes
        Simd::from_slice(&haystack[start.min(len - 16)..])
    }
}
