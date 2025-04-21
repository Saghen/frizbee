//! Greedy fallback fuzzy matching algorithm, which doesn't use Smith Waterman
//! to find the optimal alignment. Runs in linear time and used for when the Smith Waterman matrix
//! would balloon in size (due to being N * M)

use crate::r#const::*;

const DELIMITERS: [u8; 7] = [b' ', b'/', b'.', b',', b'_', b'-', b':'];

pub fn match_greedy<S1: AsRef<str>, S2: AsRef<str>>(
    needle: S1,
    haystack: S2,
) -> Option<(u16, Vec<usize>)> {
    let needle = needle.as_ref().as_bytes();
    let haystack = haystack.as_ref().as_bytes();

    let mut score = 0;
    let mut indices = vec![];
    let mut haystack_idx = 0;

    let mut delimiter_bonus_enabled = false;
    let mut previous_haystack_is_lower = false;
    let mut previous_haystack_is_delimiter = false;
    'outer: for needle_idx in 0..needle.len() {
        let needle_char = needle[needle_idx];
        let needle_is_upper = (65..=90).contains(&needle_char);
        let needle_is_lower = (97..=122).contains(&needle_char);

        let needle_lower_char = if needle_is_upper {
            needle_char + 32
        } else {
            needle_char
        };
        let needle_upper_char = if needle_is_lower {
            needle_char - 32
        } else {
            needle_char
        };

        let haystack_start_idx = haystack_idx;
        while haystack_idx <= (haystack.len() - needle.len() + needle_idx) {
            let haystack_char = haystack[haystack_idx];
            let haystack_is_delimiter = DELIMITERS.contains(&haystack_char);
            let haystack_is_upper = (65..=90).contains(&haystack_char);
            let haystack_is_lower = (97..=122).contains(&haystack_char);

            // Only enable delimiter bonus if we've seen a non-delimiter char
            if !haystack_is_delimiter {
                delimiter_bonus_enabled = true;
            }

            if needle_lower_char != haystack_char && needle_upper_char != haystack_char {
                previous_haystack_is_delimiter = delimiter_bonus_enabled && haystack_is_delimiter;
                previous_haystack_is_lower = haystack_is_lower;
                haystack_idx += 1;
                continue;
            }

            // found a match, add the scores and continue the outer loop
            score += MATCH_SCORE;

            // gap penalty
            if haystack_idx != haystack_start_idx && needle_idx != 0 {
                score = score.saturating_sub(
                    GAP_OPEN_PENALTY
                        + GAP_EXTEND_PENALTY
                            * (haystack_idx - haystack_start_idx).saturating_sub(1) as u16,
                );
            }

            // bonuses (see constant documentation for details)
            if needle_char == haystack_char {
                score += MATCHING_CASE_BONUS;
            }
            if haystack_is_upper && previous_haystack_is_lower {
                score += CAPITALIZATION_BONUS;
            }
            if haystack_idx == 0 {
                score += PREFIX_BONUS;
            }
            if previous_haystack_is_delimiter && !haystack_is_delimiter {
                score += DELIMITER_BONUS;
            }

            previous_haystack_is_delimiter = delimiter_bonus_enabled && haystack_is_delimiter;
            previous_haystack_is_lower = haystack_is_lower;

            indices.push(haystack_idx);
            haystack_idx += 1;
            continue 'outer;
        }

        // didn't find a match, so return None
        return None;
    }

    if needle == haystack {
        score += EXACT_MATCH_BONUS;
    }

    Some((score, indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAR_SCORE: u16 = MATCH_SCORE + MATCHING_CASE_BONUS;

    fn get_score(needle: &str, haystack: &str) -> u16 {
        match_greedy(needle, haystack).unwrap().0
    }

    #[test]
    fn test_score_basic() {
        assert_eq!(get_score("b", "abc"), CHAR_SCORE);
        assert_eq!(get_score("c", "abc"), CHAR_SCORE);
        assert_eq!(
            get_score("fbb", "barbazfoobarbaz"),
            CHAR_SCORE - GAP_OPEN_PENALTY - GAP_EXTEND_PENALTY + CHAR_SCORE
                - GAP_OPEN_PENALTY
                - GAP_EXTEND_PENALTY
                + CHAR_SCORE
        );
    }

    #[test]
    fn test_no_match() {
        assert_eq!(match_greedy("a", "b"), None);
        assert_eq!(match_greedy("ab", "ba"), None);
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
        assert_eq!(get_score("-", "a--bc"), CHAR_SCORE);
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
        assert_eq!(
            get_score("d", "forDist"),
            MATCH_SCORE + CAPITALIZATION_BONUS
        );
        assert_eq!(get_score("D", "forDist"), CHAR_SCORE + CAPITALIZATION_BONUS);
        assert_eq!(get_score("D", "foRDist"), CHAR_SCORE);
        assert_eq!(get_score("D", "FOR_DIST"), CHAR_SCORE + DELIMITER_BONUS);
    }

    #[test]
    fn test_score_prefix_beats_delimiter() {
        assert!(get_score("swap", "swap(test)") > get_score("swap", "iter_swap(test)"));
        assert!(get_score("_", "_private_member") > get_score("_", "public_member"));
    }
}
