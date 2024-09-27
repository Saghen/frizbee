#[allow(dead_code)]

// 128-bit SIMD with u8
// NOTE: going above 16 results in much slower performance
pub const SIMD_WIDTH: usize = 16;

// NOTE: be vary carefuly when changing these values since they affect what can fit in
// the u8 scoring without overflowing
// TODO: control if we use u8 or u16 for scoring based on the scoring values and the size
// of the needle

pub const MATCH_SCORE: u8 = 8;
pub const MISMATCH_PENALTY: u8 = 4; // -4
pub const GAP_OPEN_PENALTY: u8 = 3; // -3
pub const GAP_EXTEND_PENALTY: u8 = 1; // -1

// bonus for matching the first character of the haystack
pub const PREFIX_BONUS: u8 = 4;
// bonus for matching character after a delimiter in the haystack (e.g. space, comma, underscore, slash, etc)
pub const DELIMITER_BONUS: u8 = 2;
// bonus for matching a letter that is capitalized on the haystack
pub const CAPITALIZATION_BONUS: u8 = 2;
// bonus multiplier for the first character of the needle
pub const FIRST_CHAR_MULTIPLIER: u8 = 4;
