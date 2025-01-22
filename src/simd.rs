use crate::r#const::*;
use std::ops::Not;
use std::simd::num::SimdUint;
use std::simd::{cmp::*, SimdElement};
use std::simd::{Mask, Simd};

pub trait SimdNum<const L: usize>:
    Sized
    + Copy
    + std::fmt::Debug
    + std::simd::SimdElement
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::convert::From<u8>
    + std::convert::Into<u16>
    + std::convert::Into<usize>
    + std::cmp::PartialEq
    + std::cmp::PartialOrd
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    const ZERO: Self;
    const ZERO_VEC: Simd<Self, L>;

    // Delmiters
    const SPACE_DELIMITER: Simd<Self, L>;
    const SLASH_DELIMITER: Simd<Self, L>;
    const DOT_DELIMITER: Simd<Self, L>;
    const COMMA_DELIMITER: Simd<Self, L>;
    const UNDERSCORE_DELIMITER: Simd<Self, L>;
    const DASH_DELIMITER: Simd<Self, L>;
    const COLON_DELIMITER: Simd<Self, L>;
    const DELIMITER_BONUS: Simd<Self, L>;

    // Capitalization
    const CAPITAL_START: Simd<Self, L>;
    const CAPITAL_END: Simd<Self, L>;
    const TO_LOWERCASE_MASK: Simd<Self, L>;

    // Scoring Params
    const CAPITALIZATION_BONUS: Simd<Self, L>;
    const MATCHING_CASE_BONUS: Simd<Self, L>;

    const GAP_OPEN_PENALTY: Simd<Self, L>;
    const GAP_EXTEND_PENALTY: Simd<Self, L>;
    const MATCH_SCORE: Simd<Self, L>;
    const MISMATCH_PENALTY: Simd<Self, L>;
    const PREFIX_MATCH_SCORE: Simd<Self, L>;

    fn from_usize(n: usize) -> Self;
}

pub trait SimdVec<N: SimdNum<L>, const L: usize>:
    Sized
    + Copy
    + std::ops::Add<Output = Simd<N, L>>
    + std::ops::BitOr<Output = Simd<N, L>>
    + std::simd::cmp::SimdPartialEq<Mask = Mask<<N as SimdElement>::Mask, L>>
    + std::simd::cmp::SimdOrd
    + std::simd::num::SimdUint
where
    N: SimdNum<L>,
    N::Mask: std::simd::MaskElement,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
}

pub trait SimdMask<N: SimdNum<L>, const L: usize>:
    Sized
    + Copy
    + std::ops::Not<Output = Mask<N::Mask, L>>
    + std::ops::BitAnd<Output = Mask<N::Mask, L>>
    + std::ops::BitOr<Output = Mask<N::Mask, L>>
    + std::simd::cmp::SimdPartialEq<Mask = Mask<<N as SimdElement>::Mask, L>>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
}

macro_rules! simd_num_impl {
    ($type:ident,$($lanes:literal),+) => {
        $(
            impl SimdNum<$lanes> for $type {
                const ZERO: Self = 0;
                const ZERO_VEC: Simd<Self, $lanes> = Simd::from_array([0; $lanes]);

                const SPACE_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b' ' as $type; $lanes]);
                const SLASH_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b'/' as $type; $lanes]);
                const DOT_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b'.' as $type; $lanes]);
                const COMMA_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b',' as $type; $lanes]);
                const UNDERSCORE_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b'_' as $type; $lanes]);
                const DASH_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b'-' as $type; $lanes]);
                const COLON_DELIMITER: Simd<Self, $lanes> = Simd::from_array([b':' as $type; $lanes]);
                const DELIMITER_BONUS: Simd<Self, $lanes> = Simd::from_array([DELIMITER_BONUS as $type; $lanes]);

                const CAPITAL_START: Simd<Self, $lanes> = Simd::from_array([b'A' as $type; $lanes]);
                const CAPITAL_END: Simd<Self, $lanes> = Simd::from_array([b'Z' as $type; $lanes]);
                const TO_LOWERCASE_MASK: Simd<Self, $lanes> = Simd::from_array([0x20; $lanes]);

                const CAPITALIZATION_BONUS: Simd<Self, $lanes> = Simd::from_array([CAPITALIZATION_BONUS as $type; $lanes]);
                const MATCHING_CASE_BONUS: Simd<Self, $lanes> = Simd::from_array([MATCHING_CASE_BONUS as $type; $lanes]);

                const GAP_OPEN_PENALTY: Simd<Self, $lanes> = Simd::from_array([GAP_OPEN_PENALTY as $type; $lanes]);
                const GAP_EXTEND_PENALTY: Simd<Self, $lanes> = Simd::from_array([GAP_EXTEND_PENALTY as $type; $lanes]);
                const MATCH_SCORE: Simd<Self, $lanes> = Simd::from_array([MATCH_SCORE as $type; $lanes]);
                const MISMATCH_PENALTY: Simd<Self, $lanes> = Simd::from_array([MISMATCH_PENALTY as $type; $lanes]);
                const PREFIX_MATCH_SCORE: Simd<Self, $lanes> = Simd::from_array([(MATCH_SCORE + PREFIX_BONUS) as $type; $lanes]);

                #[inline(always)]
                fn from_usize(n: usize) -> Self {
                    n as $type
                }
            }
            impl SimdVec<$type, $lanes> for Simd<$type, $lanes> {}
            impl SimdMask<$type, $lanes> for Mask<<$type as SimdElement>::Mask, $lanes> {}
        )+
    };
}
simd_num_impl!(u8, 1, 2, 4, 8, 16, 32, 64);
simd_num_impl!(u16, 1, 2, 4, 8, 16, 32);

pub fn smith_waterman<N, const W: usize, const L: usize>(
    needle: &str,
    haystacks: &[&str; L],
) -> ([u16; L], [u16; L])
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    Simd<N, L>: SimdVec<N, L>,
    Mask<N::Mask, L>: SimdMask<N, L>,
{
    let needle_str = needle;
    let needle = needle.as_bytes();
    let needle_len = needle.len();
    assert!(needle_len > 0);
    let haystack_len = haystacks.iter().map(|&x| x.len()).max().unwrap();
    assert!(haystack_len <= W);

    // Convert haystacks to a static array of bytes chunked for SIMD
    let mut haystack: [[N; L]; W] = [[N::ZERO; L]; W];
    for (str_idx, &haystack_str) in haystacks.iter().enumerate() {
        for (char_idx, &haystack_char) in haystack_str.as_bytes().iter().enumerate() {
            haystack[char_idx][str_idx] = N::from(haystack_char);
        }
    }

    // State
    let mut score_matrix = vec![[N::ZERO_VEC; W]; needle.len()];
    let mut all_time_max_score = N::ZERO_VEC;

    for i in 0..needle_len {
        let prev_col_scores = if i > 0 {
            score_matrix[i - 1]
        } else {
            [N::ZERO_VEC; W]
        };
        let curr_col_scores = &mut score_matrix[i];

        let mut up_score_simd = N::ZERO_VEC;
        let mut up_gap_penalty_mask = Mask::splat(true);

        let needle_char = Simd::splat(N::from(needle[i]));
        let needle_cased_mask: Mask<N::Mask, L> =
            needle_char.simd_ge(N::CAPITAL_START) & needle_char.simd_le(N::CAPITAL_END);
        let needle_char = needle_char | needle_cased_mask.select(N::TO_LOWERCASE_MASK, N::ZERO_VEC);

        let mut left_gap_penalty_mask = Mask::splat(true);
        let mut delimiter_bonus_enabled_mask = Mask::splat(false);
        let mut is_delimiter_mask = Mask::splat(false);

        for j in 0..haystack_len {
            let is_prefix = j == 0;

            // Load chunk and remove casing
            let cased_haystack_simd = Simd::from_slice(&haystack[j]);
            let capital_mask: Mask<N::Mask, L> = cased_haystack_simd.simd_ge(N::CAPITAL_START)
                & cased_haystack_simd.simd_le(N::CAPITAL_END);
            let haystack_simd =
                cased_haystack_simd | capital_mask.select(N::TO_LOWERCASE_MASK, N::ZERO_VEC);

            let matched_casing_mask: Mask<N::Mask, L> = needle_cased_mask.simd_eq(capital_mask);

            // Give a bonus for prefix matches
            let match_score = if is_prefix {
                N::PREFIX_MATCH_SCORE
            } else {
                N::MATCH_SCORE
            };

            // Calculate diagonal (match/mismatch) scores
            let diag = if is_prefix {
                N::ZERO_VEC
            } else {
                prev_col_scores[j - 1]
            };
            let match_mask: Mask<N::Mask, L> = needle_char.simd_eq(haystack_simd);
            let diag_score: Simd<N, L> = match_mask.select(
                diag + match_score
                    + (is_delimiter_mask & delimiter_bonus_enabled_mask).select(N::DELIMITER_BONUS, N::ZERO_VEC)
                    // ignore capitalization on the prefix
                    + if is_prefix { capital_mask.select(N::CAPITALIZATION_BONUS, N::ZERO_VEC) } else { N::ZERO_VEC }
                    + matched_casing_mask.select(N::MATCHING_CASE_BONUS, N::ZERO_VEC),
                diag.saturating_sub(N::MISMATCH_PENALTY),
            );

            // Load and calculate up scores (skipping char in haystack)
            let up_gap_penalty =
                up_gap_penalty_mask.select(N::GAP_OPEN_PENALTY, N::GAP_EXTEND_PENALTY);
            let up_score = up_score_simd.saturating_sub(up_gap_penalty);

            // Load and calculate left scores (skipping char in needle)
            let left = prev_col_scores[j];
            let left_gap_penalty =
                left_gap_penalty_mask.select(N::GAP_OPEN_PENALTY, N::GAP_EXTEND_PENALTY);
            let left_score = left.saturating_sub(left_gap_penalty);

            // Calculate maximum scores
            let max_score = diag_score.simd_max(up_score).simd_max(left_score);

            // Update gap penalty mask
            let diag_mask: Mask<N::Mask, L> = max_score.simd_eq(diag_score);
            up_gap_penalty_mask = max_score.simd_ne(up_score) | diag_mask;
            left_gap_penalty_mask = max_score.simd_ne(left_score) | diag_mask;

            // Update delimiter masks
            is_delimiter_mask = N::SPACE_DELIMITER.simd_eq(haystack_simd)
                | N::SLASH_DELIMITER.simd_eq(haystack_simd)
                | N::DOT_DELIMITER.simd_eq(haystack_simd)
                | N::COMMA_DELIMITER.simd_eq(haystack_simd)
                | N::UNDERSCORE_DELIMITER.simd_eq(haystack_simd)
                | N::DASH_DELIMITER.simd_eq(haystack_simd)
                | N::SPACE_DELIMITER.simd_eq(haystack_simd);
            // Only enable delimiter bonus if we've seen a non-delimiter char
            delimiter_bonus_enabled_mask |= is_delimiter_mask.not();

            // Store the scores for the next iterations
            up_score_simd = max_score;
            curr_col_scores[j] = max_score;

            // Store the maximum score across all runs
            all_time_max_score = all_time_max_score.simd_max(max_score);
        }
    }

    let mut max_scores_vec = [0u16; L];
    for i in 0..L {
        max_scores_vec[i] = all_time_max_score[i].into();
        if haystacks[i] == needle_str {
            max_scores_vec[i] += EXACT_MATCH_BONUS as u16;
        }
    }

    (max_scores_vec, typos_from_score_matrix(score_matrix))
}

pub fn typos_from_score_matrix<N, const W: usize, const L: usize>(
    score_matrix: Vec<[Simd<N, L>; W]>,
) -> [u16; L]
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    Simd<N, L>: SimdVec<N, L>,
    Mask<N::Mask, L>: SimdMask<N, L>,
{
    let mut typo_count = [0u16; L];
    let mut scores = N::ZERO_VEC;
    let mut positions = N::ZERO_VEC;

    // Get the starting position by looking at the last column
    // (last character of the needle)
    let last_column = score_matrix.last().unwrap();
    for idx in 0..W {
        let row_scores = last_column[idx];
        let row_max_mask: Mask<N::Mask, L> = row_scores.simd_gt(scores);
        scores = row_max_mask.select(row_scores, scores);
        positions = row_max_mask.select(Simd::splat(N::from_usize(idx)), positions);
    }

    // Traceback and store the matched indices
    for (idx, &row_idx) in positions.to_array().iter().enumerate() {
        let mut col_idx = score_matrix.len() - 1;
        let mut row_idx: usize = row_idx.into();
        let mut score = scores[idx];

        // NOTE: row_idx = 0 or col_idx = 0 will always have a score of 0
        while col_idx > 0 {
            // Must be moving left
            if row_idx == 0 {
                typo_count[idx] += 1;
                col_idx -= 1;
                continue;
            }

            // Gather up the scores for all possible paths
            let diag = score_matrix[col_idx - 1][row_idx - 1][idx];
            let left = score_matrix[col_idx - 1][row_idx][idx];
            let up = score_matrix[col_idx][row_idx - 1][idx];

            // Match or mismatch
            if diag >= left && diag >= up {
                // Must be a mismatch
                if diag >= score {
                    typo_count[idx] += 1;
                }
                row_idx -= 1;
                col_idx -= 1;
                score = diag;
            // Skipped character in needle
            } else if left >= up {
                typo_count[idx] += 1;
                col_idx -= 1;
                score = left;
            // Skipped character in haystack
            } else {
                row_idx -= 1;
                score = up;
            }
        }

        // HACK: Compensate for the last column being a typo
        if col_idx == 0 && score == N::ZERO {
            typo_count[idx] += 1;
        }
    }

    typo_count
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAR_SCORE: u8 = MATCH_SCORE + MATCHING_CASE_BONUS;

    fn get_score(needle: &str, haystack: &str) -> u8 {
        smith_waterman::<u8, 16, 1>(needle, &[haystack; 1]).0[0] as u8
    }

    fn get_typos(needle: &str, haystack: &str) -> u16 {
        smith_waterman::<u8, 4, 1>(needle, &[haystack; 1]).1[0]
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
            CHAR_SCORE + EXACT_MATCH_BONUS as u8 + PREFIX_BONUS
        );
        assert_eq!(
            get_score("abc", "abc"),
            3 * CHAR_SCORE + EXACT_MATCH_BONUS as u8 + PREFIX_BONUS
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
        assert_eq!(get_score("-", "a-bc"), CHAR_SCORE);
        assert_eq!(get_score("-", "a--bc"), CHAR_SCORE + DELIMITER_BONUS);
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
        assert_eq!(get_score("D", "forDist"), CHAR_SCORE);
    }

    #[test]
    fn test_score_prefix_beats_delimiter() {
        assert!(get_score("swap", "swap(test)") > get_score("swap", "iter_swap(test)"),);
    }
}
