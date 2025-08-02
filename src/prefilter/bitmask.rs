use std::simd::{cmp::SimdPartialOrd, num::SimdUint, Simd};

use multiversion::multiversion;

const LANES: usize = 8;

/// Converts a string to a u64 where each bit represents the existence of a character in the ASCII
/// range `[33, 90]`. To tell if two strings are likely to match, we perform a bitwise XOR between
/// the two bitmasks. The number of 1s in the resulting bitmask indicates the number of characters
/// in the needle which are not in the haystack and the number of characters in the haystack which
/// are not in the needle.
#[multiversion(targets(
    // x86-64-v4 without lahfsahf
    "x86_64+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
    // x86-64-v3 without lahfsahf
    "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
    // x86-64-v2 without lahfsahf
    "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
))]
pub fn string_to_bitmask(s: &[u8]) -> u64 {
    let mut mask: u64 = 0;

    let zero = Simd::splat(0);
    let zero_wide = Simd::splat(0);
    let one = Simd::splat(1);
    let to_upperacse = Simd::splat(0x20);

    let mut i = 0;
    while i < s.len() {
        let simd_chunk = Simd::<u8, LANES>::load_or_default(&s[i..(i + LANES).min(s.len())]);

        // Convert to uppercase
        let is_lower =
            simd_chunk.simd_ge(Simd::splat(b'a')) & simd_chunk.simd_le(Simd::splat(b'z'));
        let simd_upper = simd_chunk - is_lower.select(to_upperacse, zero);

        // Check if characters are in the valid range [33, 90]
        let in_range =
            simd_upper.simd_ge(Simd::splat(33u8)) & simd_upper.simd_le(Simd::splat(90u8));

        // Compute indices
        let indices = in_range.cast::<i64>().select(
            one << (simd_upper - Simd::splat(33u8)).cast::<u64>(),
            zero_wide,
        );

        mask |= indices.reduce_or();

        i += LANES;
    }

    mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive() {
        assert_eq!(
            string_to_bitmask("ABC".as_bytes()),
            string_to_bitmask("abc".as_bytes())
        );
    }

    #[test]
    fn test_letters() {
        assert_eq!(
            string_to_bitmask("abC".as_bytes()),
            0b0000000000000000000000000000011100000000000000000000000000000000
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(
            string_to_bitmask("123".as_bytes()),
            0b00000000000000000000000000000000000000000001110000000000000000
        );
    }

    #[test]
    fn test_symbols() {
        assert_eq!(
            string_to_bitmask("!\"#$%&'()*+,-./".as_bytes()),
            0b00000000000000000000000000000000000000000000000111111111111111
        );
    }
}
