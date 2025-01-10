#[allow(dead_code)]

// 128-bit SIMD with u16
// NOTE: going above 128 bit results in much slower performance
pub const SIMD_WIDTH: usize = 16;

// NOTE: be vary carefuly when changing these values since they affect what can fit in
// the u8 scoring without overflowing

pub const MATCH_SCORE: u8 = 7;
pub const MISMATCH_PENALTY: u8 = 4; // -4
pub const GAP_OPEN_PENALTY: u8 = 3; // -3
pub const GAP_EXTEND_PENALTY: u8 = 1; // -1

// bonus for matching the first character of the haystack
pub const PREFIX_BONUS: u8 = 6;
// bonus for matching character after a delimiter in the haystack (e.g. space, comma, underscore, slash, etc)
pub const DELIMITER_BONUS: u8 = 4;
// bonus for matching a letter that is capitalized on the haystackA
// FIXME: temporarily disabled until we can apply only when the char before the capital is
// lowercase
pub const CAPITALIZATION_BONUS: u8 = 0;
pub const MATCHING_CASE_BONUS: u8 = 1;
// bonus multiplier for the first character of the needle
pub const FIRST_CHAR_MULTIPLIER: u8 = 1;
// bonus for haystack == needle
pub const EXACT_MATCH_BONUS: u8 = 4;

// TODO: bonus for a full continuous match without gaps?
