use crate::r#const::*;

pub fn smith_waterman<const W: usize>(needle: &str, haystack: &str) -> u16 {
    assert!(haystack.len() <= W);

    let needle = needle.as_bytes();
    let haystack = haystack.as_bytes();

    // State
    let mut score_matrix = vec![[0; W]; needle.len()];
    let mut left_gap_penalty_masks = [true; W];
    let mut all_time_max_score = 0;
    let mut all_time_max_score_row = 0;
    let mut all_time_max_score_col = 0;

    // Delimiters
    let space_delimiter = b' ';
    let slash_delimiter = b'/';
    let dot_delimiter = b'.';
    let comma_delimiter = b',';
    let underscore_delimiter = b'_';
    let dash_delimiter = b'-';
    let colon_delimiter = b':';
    let delimiter_bonus = DELIMITER_BONUS;

    // Capitalization
    let capitalization_bonus = CAPITALIZATION_BONUS;
    let matching_casing_bonus = MATCHING_CASE_BONUS;

    // Scoring params
    let gap_open_penalty = GAP_OPEN_PENALTY;
    let gap_extend_penalty = GAP_EXTEND_PENALTY;

    let match_score = MATCH_SCORE;
    let mismatch_score = MISMATCH_PENALTY;
    let prefix_match_score = MATCH_SCORE + PREFIX_BONUS;

    for i in 0..needle.len() {
        let prev_col_scores = if i == 0 { [0; W] } else { score_matrix[i - 1] };
        let curr_col_scores = &mut score_matrix[i];

        let mut up_score_simd: u8 = 0;
        let mut up_gap_penalty_mask = true;

        let needle_char = needle[i];
        let needle_cased_mask = needle_char.is_ascii_uppercase();
        let needle_char = needle_char.to_ascii_lowercase();

        let mut delimiter_bonus_enabled_mask = false;
        let mut is_delimiter_mask = false;

        for j in 0..haystack.len() {
            let prefix_mask = j == 0;

            // Load chunk and remove casing
            let cased_haystack_simd = haystack[j];
            let capital_mask = cased_haystack_simd.is_ascii_uppercase();
            let haystack_simd = cased_haystack_simd.to_ascii_lowercase();

            // Give a bonus for prefix matches
            let match_score = if prefix_mask {
                prefix_match_score
            } else {
                match_score
            };

            // Calculate diagonal (match/mismatch) scores
            let diag = if j > 0 { prev_col_scores[j - 1] } else { 0 };
            let diag_score = if needle_char == haystack_simd {
                diag + match_score
                    + if is_delimiter_mask && delimiter_bonus_enabled_mask { delimiter_bonus } else { 0 }
                    // XOR with prefix mask to ignore capitalization on the prefix
                    + if capital_mask && !prefix_mask { capitalization_bonus } else { 0 }
                    + if needle_cased_mask == capital_mask { matching_casing_bonus } else { 0 }
            } else {
                diag.saturating_sub(mismatch_score)
            };

            // Load and calculate up scores (skipping char in haystack)
            let up_gap_penalty = if up_gap_penalty_mask {
                gap_open_penalty
            } else {
                gap_extend_penalty
            };
            let up_score = up_score_simd.saturating_sub(up_gap_penalty);

            // Load and calculate left scores (skipping char in needle)
            let left = prev_col_scores[j];
            let left_gap_penalty = if left_gap_penalty_masks[j] {
                gap_open_penalty
            } else {
                gap_extend_penalty
            };
            let left_score = left.saturating_sub(left_gap_penalty);

            // Calculate maximum scores
            let max_score = diag_score.max(up_score).max(left_score);

            // Update gap penalty mask
            let diag_mask = max_score == diag_score;
            up_gap_penalty_mask = max_score != up_score || diag_mask;
            left_gap_penalty_masks[j] = max_score != left_score || diag_mask;

            // Update delimiter mask
            is_delimiter_mask = space_delimiter == haystack_simd
                || slash_delimiter == haystack_simd
                || dot_delimiter == haystack_simd
                || comma_delimiter == haystack_simd
                || underscore_delimiter == haystack_simd
                || dash_delimiter == haystack_simd
                || colon_delimiter == haystack_simd;
            // Only enable delimiter bonus if we've seen a non-delimiter char
            delimiter_bonus_enabled_mask |= !is_delimiter_mask;

            // Store the scores for the next iterations
            up_score_simd = max_score;
            curr_col_scores[j] = max_score;

            // Store the maximum score across all runs
            // TODO: shouldn't we only care about the max score of the final column?
            // since we want to match the entire needle to see how many typos there are
            (all_time_max_score_col, all_time_max_score_row) = if all_time_max_score_col < max_score
            {
                // TODO: must guarantee that needle.len() < 2 ** 8
                ((i + 1) as u8, (j + 1) as u8)
            } else {
                (all_time_max_score_col, all_time_max_score_row)
            };

            all_time_max_score = all_time_max_score.max(max_score);
        }
    }

    if haystack == needle {
        all_time_max_score as u16 + EXACT_MATCH_BONUS
    } else {
        all_time_max_score as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAR_SCORE: u8 = MATCH_SCORE + MATCHING_CASE_BONUS;

    fn run_single(needle: &str, haystack: &str) -> u8 {
        smith_waterman::<16>(needle, haystack) as u8
    }

    #[test]
    fn test_basic() {
        assert_eq!(run_single("b", "abc"), CHAR_SCORE);
        assert_eq!(run_single("c", "abc"), CHAR_SCORE);
    }

    #[test]
    fn test_prefix() {
        assert_eq!(run_single("a", "abc"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(run_single("a", "aabc"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(run_single("a", "babc"), CHAR_SCORE);
    }

    #[test]
    fn test_exact_match() {
        assert_eq!(
            run_single("a", "a"),
            CHAR_SCORE + EXACT_MATCH_BONUS as u8 + PREFIX_BONUS
        );
        assert_eq!(
            run_single("abc", "abc"),
            3 * CHAR_SCORE + EXACT_MATCH_BONUS as u8 + PREFIX_BONUS
        );
        assert_eq!(run_single("ab", "abc"), 2 * CHAR_SCORE + PREFIX_BONUS);
        // assert_eq!(run_single("abc", "ab"), 2 * CHAR_SCORE + PREFIX_BONUS);
    }

    #[test]
    fn test_delimiter() {
        assert_eq!(run_single("b", "a-b"), CHAR_SCORE + DELIMITER_BONUS);
        assert_eq!(run_single("a", "a-b-c"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(run_single("b", "a--b"), CHAR_SCORE + DELIMITER_BONUS);
        assert_eq!(run_single("c", "a--bc"), CHAR_SCORE);
        assert_eq!(run_single("a", "-a--bc"), CHAR_SCORE);
        assert_eq!(run_single("-", "a-bc"), CHAR_SCORE);
        assert_eq!(run_single("-", "a--bc"), CHAR_SCORE + DELIMITER_BONUS);
    }

    #[test]
    fn test_affine_gap() {
        assert_eq!(
            run_single("test", "Uterst"),
            CHAR_SCORE * 4 - GAP_OPEN_PENALTY
        );
        assert_eq!(
            run_single("test", "Uterrst"),
            CHAR_SCORE * 4 - GAP_OPEN_PENALTY - GAP_EXTEND_PENALTY
        );
    }

    #[test]
    fn test_capital_bonus() {
        assert_eq!(run_single("a", "A"), MATCH_SCORE + PREFIX_BONUS);
        assert_eq!(run_single("A", "Aa"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(run_single("D", "forDist"), CHAR_SCORE);
    }

    #[test]
    fn test_prefix_beats_delimiter() {
        assert!(run_single("swap", "swap(test)") > run_single("swap", "iter_swap(test)"),);
    }
}
