use std::simd::{cmp::SimdPartialOrd, num::SimdUint, Simd};

pub fn string_to_bitmask(s: &[u8]) -> u64 {
    let mut mask: u64 = 0;
    for c in s {
        let c = c.to_ascii_uppercase();
        if (33..=90).contains(&c) {
            mask |= 1 << (c - 33);
        }
    }
    mask
}

const LANES: usize = 8;
#[inline(always)]
pub fn string_to_bitmask_simd(s: &[u8]) -> u64 {
    let mut mask: u64 = 0;

    let zero = Simd::splat(0);
    let zero_wide = Simd::splat(0);
    let one = Simd::splat(1);
    let to_upperacse = Simd::splat(0x20);

    let mut i = 0;
    while i + LANES <= s.len() {
        let simd_chunk = Simd::<u8, LANES>::from_slice(&s[i..i + LANES]);

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

    // Process remaining characters
    for &c in s[i..s.len()].iter() {
        let c = c.to_ascii_uppercase();
        if (33..=90).contains(&c) {
            mask |= 1 << (c - 33);
        }
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

    #[test]
    fn test_simd() {
        assert_eq!(
            string_to_bitmask_simd("abc".as_bytes()),
            string_to_bitmask("abc".as_bytes())
        );
    }
}
