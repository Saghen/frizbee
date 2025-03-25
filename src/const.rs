// 128-bit SIMD with u16
// NOTE: going above 128 bit results in much slower performance unless AVX2 is enabled at compile
// time
pub const SIMD_WIDTH: usize = 16;

// NOTE: be very careful when changing these values since they affect what can fit in
// the u8 scoring without overflowing

pub const MATCH_SCORE: u16 = 6;
pub const MISMATCH_PENALTY: u16 = 4; // -4
pub const GAP_OPEN_PENALTY: u16 = 3; // -3
pub const GAP_EXTEND_PENALTY: u16 = 1; // -1

// bonus for matching the first character of the haystack
pub const PREFIX_BONUS: u16 = 6;
// bonus for matching character after a delimiter in the haystack (e.g. space, comma, underscore, slash, etc)
pub const DELIMITER_BONUS: u16 = 4;
// bonus for matching a letter that is capitalized on the haystack, if the character before it was lowercase
pub const CAPITALIZATION_BONUS: u16 = 4;
pub const MATCHING_CASE_BONUS: u16 = 2;
// bonus for haystack == needle
pub const EXACT_MATCH_BONUS: u16 = 4;
