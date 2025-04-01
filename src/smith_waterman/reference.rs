#![allow(dead_code)]

use crate::r#const::*;

pub fn smith_waterman<const W: usize>(needle: &str, haystack: &str) -> (u16, u16) {
    assert!(haystack.len() <= W);

    let needle = needle.as_bytes();
    let haystack = haystack.as_bytes();

    // State
    let mut score_matrix = vec![[0; W]; needle.len()];
    let mut all_time_max_score = 0;

    for i in 0..needle.len() {
        let prev_col_scores = if i > 0 { score_matrix[i - 1] } else { [0; W] };
        let curr_col_scores = &mut score_matrix[i];

        let mut up_score_simd: u16 = 0;
        let mut up_gap_penalty_mask = true;

        let needle_char = needle[i];
        let needle_cased_mask = needle_char.is_ascii_uppercase();
        let needle_char = needle_char.to_ascii_lowercase();

        let mut left_gap_penalty_mask = true;
        let mut delimiter_bonus_enabled_mask = false;
        let mut prev_is_delimiter_mask = false;

        for j in 0..haystack.len() {
            let is_prefix = j == 0;

            // Load chunk and remove casing
            let cased_haystack_simd = haystack[j];
            let capital_mask = cased_haystack_simd.is_ascii_uppercase();
            let haystack_simd = cased_haystack_simd.to_ascii_lowercase();

            let is_delimiter_mask =
                [b' ', b'/', b'.', b',', b'_', b'-', b':'].contains(&haystack_simd);
            let matched_casing_mask = needle_cased_mask == capital_mask;

            // Give a bonus for prefix matches
            let match_score = if is_prefix {
                MATCH_SCORE + PREFIX_BONUS
            } else {
                MATCH_SCORE
            };

            // Calculate diagonal (match/mismatch) scores
            let diag = if is_prefix { 0 } else { prev_col_scores[j - 1] };
            let match_mask = needle_char == haystack_simd;
            let diag_score = if match_mask {
                diag + match_score
                    + if prev_is_delimiter_mask && delimiter_bonus_enabled_mask && !is_delimiter_mask { DELIMITER_BONUS } else { 0 }
                    // ignore capitalization on the prefix
                    + if !is_prefix && capital_mask { CAPITALIZATION_BONUS } else { 0 }
                    + if matched_casing_mask { MATCHING_CASE_BONUS } else { 0 }
            } else {
                diag.saturating_sub(MISMATCH_PENALTY)
            };

            // Load and calculate up scores (skipping char in haystack)
            let up_gap_penalty = if up_gap_penalty_mask {
                GAP_OPEN_PENALTY
            } else {
                GAP_EXTEND_PENALTY
            };
            let up_score = up_score_simd.saturating_sub(up_gap_penalty);

            // Load and calculate left scores (skipping char in needle)
            let left = prev_col_scores[j];
            let left_gap_penalty = if left_gap_penalty_mask {
                GAP_OPEN_PENALTY
            } else {
                GAP_EXTEND_PENALTY
            };
            let left_score = left.saturating_sub(left_gap_penalty);

            // Calculate maximum scores
            let max_score = diag_score.max(up_score).max(left_score);

            // Update gap penalty mask
            let diag_mask = max_score == diag_score;
            up_gap_penalty_mask = max_score != up_score || diag_mask;
            left_gap_penalty_mask = max_score != left_score || diag_mask;

            // Update delimiter mask
            prev_is_delimiter_mask = is_delimiter_mask;
            // Only enable delimiter bonus if we've seen a non-delimiter char
            delimiter_bonus_enabled_mask |= !prev_is_delimiter_mask;

            // Store the scores for the next iterations
            up_score_simd = max_score;
            curr_col_scores[j] = max_score;

            // Store the maximum score across all runs
            all_time_max_score = all_time_max_score.max(max_score);
        }
    }

    let max_score = if haystack == needle {
        all_time_max_score + EXACT_MATCH_BONUS
    } else {
        all_time_max_score
    };

    (max_score, typos_from_score_matrix(score_matrix))
}

pub fn typos_from_score_matrix<const W: usize>(score_matrix: Vec<[u16; W]>) -> u16 {
    let mut typo_count = 0;
    let mut score = 0;
    let mut positions = 0;

    // Get the starting position by looking at the last column
    // (last character of the needle)
    let last_column = score_matrix.last().unwrap();
    for idx in 0..W {
        let row_score = last_column[idx];
        if row_score > score {
            score = row_score;
            positions = idx;
        }
    }

    // Traceback and store the matched indices
    // for (idx, &row_idx) in positions.to_array().iter().enumerate() {
    let mut col_idx = score_matrix.len() - 1;
    let mut row_idx: usize = positions;

    // NOTE: row_idx = 0 or col_idx = 0 will always have a score of 0
    while col_idx > 0 {
        // Must be moving left
        if row_idx == 0 {
            typo_count += 1;
            col_idx -= 1;
            continue;
        }

        // Gather up the scores for all possible paths
        let diag = score_matrix[col_idx - 1][row_idx - 1];
        let left = score_matrix[col_idx - 1][row_idx];
        let up = score_matrix[col_idx][row_idx - 1];

        // Match or mismatch
        if diag >= left && diag >= up {
            // Must be a mismatch
            if diag >= score {
                typo_count += 1;
            }
            row_idx -= 1;
            col_idx -= 1;
            score = diag;
        // Skipped character in needle
        } else if left >= up {
            typo_count += 1;
            col_idx -= 1;
            score = left;
        // Skipped character in haystack
        } else {
            row_idx -= 1;
            score = up;
        }
    }

    // HACK: Compensate for the last column being a typo
    if col_idx == 0 && score == 0 {
        typo_count += 1;
    }

    typo_count
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAR_SCORE: u16 = MATCH_SCORE + MATCHING_CASE_BONUS;

    fn get_score(needle: &str, haystack: &str) -> u16 {
        smith_waterman::<16>(needle, haystack).0
    }

    fn get_typos(needle: &str, haystack: &str) -> u16 {
        smith_waterman::<4>(needle, haystack).1
    }

    #[test]
    fn test_score_basic() {
        assert_eq!(get_score("b", "abc"), CHAR_SCORE);
        assert_eq!(get_score("c", "abc"), CHAR_SCORE);
    }

    #[test]
    fn test_typos_basic() {
        assert_eq!(get_typos("a", "abc"), 0);
        assert_eq!(get_typos("b", "abc"), 0);
        assert_eq!(get_typos("c", "abc"), 0);
        assert_eq!(get_typos("ac", "abc"), 0);

        assert_eq!(get_typos("d", "abc"), 1);
        assert_eq!(get_typos("da", "abc"), 1);
        assert_eq!(get_typos("dc", "abc"), 1);
        assert_eq!(get_typos("ad", "abc"), 1);
        assert_eq!(get_typos("adc", "abc"), 1);
        assert_eq!(get_typos("add", "abc"), 2);
        assert_eq!(get_typos("ddd", "abc"), 3);
        assert_eq!(get_typos("ddd", ""), 3);
        assert_eq!(get_typos("d", ""), 1);
    }

    #[test]
    fn test_score_prefix() {
        assert_eq!(get_score("a", "abc"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("a", "aabc"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("a", "babc"), CHAR_SCORE);
    }

    #[test]
    fn test_score_exact_match() {
        assert_eq!(
            get_score("a", "a"),
            CHAR_SCORE + EXACT_MATCH_BONUS + PREFIX_BONUS
        );
        assert_eq!(
            get_score("abc", "abc"),
            3 * CHAR_SCORE + EXACT_MATCH_BONUS + PREFIX_BONUS
        );
        assert_eq!(get_score("ab", "abc"), 2 * CHAR_SCORE + PREFIX_BONUS);
        // assert_eq!(run_single("abc", "ab"), 2 * CHAR_SCORE + PREFIX_BONUS);
    }

    #[test]
    fn test_score_delimiter() {
        assert_eq!(get_score("b", "a-b"), CHAR_SCORE + DELIMITER_BONUS);
        assert_eq!(get_score("a", "a-b-c"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("b", "a--b"), CHAR_SCORE + DELIMITER_BONUS);
        assert_eq!(get_score("c", "a--bc"), CHAR_SCORE);
        assert_eq!(get_score("a", "-a--bc"), CHAR_SCORE);
    }

    #[test]
    fn test_score_no_delimiter_for_delimiter_chars() {
        assert_eq!(get_score("-", "a-bc"), CHAR_SCORE);
        assert_eq!(get_score("-", "a--bc"), CHAR_SCORE);
        assert!(get_score("a_b", "a_bb") > get_score("a_b", "a__b"));
    }

    #[test]
    fn test_score_affine_gap() {
        assert_eq!(
            get_score("test", "Uterst"),
            CHAR_SCORE * 4 - GAP_OPEN_PENALTY
        );
        assert_eq!(
            get_score("test", "Uterrst"),
            CHAR_SCORE * 4 - GAP_OPEN_PENALTY - GAP_EXTEND_PENALTY
        );
    }

    #[test]
    fn test_score_capital_bonus() {
        assert_eq!(get_score("a", "A"), MATCH_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("A", "Aa"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("D", "forDist"), CHAR_SCORE + CAPITALIZATION_BONUS);
        assert_eq!(get_score("D", "foRDist"), CHAR_SCORE + CAPITALIZATION_BONUS);
    }

    #[test]
    fn test_score_prefix_beats_delimiter() {
        assert!(get_score("swap", "swap(test)") > get_score("swap", "iter_swap(test)"),);
    }
}
