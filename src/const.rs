pub const MATCH_SCORE: u16 = 12; // Score for a match
pub const MISMATCH_PENALTY: u16 = 8; // Penalty for a mismatch (substitution)
pub const GAP_OPEN_PENALTY: u16 = 3; // Penalty for opening a gap (deletion/insertion)
pub const GAP_EXTEND_PENALTY: u16 = 1; // Penalty for extending a gap (deletion/insertion)

pub const PREFIX_BONUS: u16 = 12; // Bonus for matching the first character of the haystack
pub const DELIMITER_BONUS: u16 = 8; // Bonus for matching _after_ a delimiter character (e.g. "hw" on "hello_world", will give a bonus on "w")
pub const CAPITALIZATION_BONUS: u16 = 8; // Bonus for matching a capital letter after a lowercase letter (e.g. "b" on "fooBar" will receive a bonus on "B")
pub const MATCHING_CASE_BONUS: u16 = 4; // Bonus for matching the case of the needle (e.g. "WorLd" on "WoRld" will receive a bonus on "W", "o", "d")
pub const EXACT_MATCH_BONUS: u16 = 8; // Bonus for matching the exact needle (e.g. "foo" on "foo" will receive the bonus)
