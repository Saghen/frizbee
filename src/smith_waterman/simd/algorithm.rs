use crate::r#const::*;
use std::ops::Not;
use std::simd::cmp::*;
use std::simd::num::SimdUint;
use std::simd::{Mask, Simd};

use super::{HaystackChar, NeedleChar, SimdMask, SimdNum, SimdVec};

#[inline(always)]
pub(crate) fn smith_waterman_inner<N, const L: usize>(
    start: usize,
    end: usize,
    needle_char: NeedleChar<N, L>,
    haystack: &[HaystackChar<N, L>],
    prev_score_col: Option<&[Simd<N, L>]>,
    curr_score_col: &mut [Simd<N, L>],
) where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    Simd<N, L>: SimdVec<N, L>,
    Mask<N::Mask, L>: SimdMask<N, L>,
{
    let mut up_score_simd = N::ZERO_VEC;
    let mut up_gap_penalty_mask = Mask::splat(true);
    let mut left_gap_penalty_mask = Mask::splat(true);
    let mut delimiter_bonus_enabled_mask = Mask::splat(false);

    for haystack_idx in start..end {
        let haystack_char = haystack[haystack_idx];

        let (diag, left) = if haystack_idx == 0 {
            (N::ZERO_VEC, N::ZERO_VEC)
        } else {
            prev_score_col
                .map(|c| (c[haystack_idx - 1], c[haystack_idx]))
                .unwrap_or((N::ZERO_VEC, N::ZERO_VEC))
        };

        // Calculate diagonal (match/mismatch) scores
        let match_mask: Mask<N::Mask, L> = needle_char.lowercase.simd_eq(haystack_char.lowercase);
        let matched_casing_mask: Mask<N::Mask, L> = needle_char
            .is_capital_mask
            .simd_eq(haystack_char.is_capital_mask);
        let diag_score: Simd<N, L> = match_mask.select(
            diag + matched_casing_mask.select(N::MATCHING_CASE_BONUS, N::ZERO_VEC)
                + if haystack_idx > 0 {
                    let prev_haystack_char = haystack[haystack_idx - 1];

                    // ignore capitalization on the prefix
                    let capitalization_bonus_mask: Mask<N::Mask, L> =
                        haystack_char.is_capital_mask & prev_haystack_char.is_lower_mask;
                    let capitalization_bonus =
                        capitalization_bonus_mask.select(N::CAPITALIZATION_BONUS, N::ZERO_VEC);

                    let delimiter_bonus_mask: Mask<N::Mask, L> = prev_haystack_char
                        .is_delimiter_mask
                        & delimiter_bonus_enabled_mask
                        & !haystack_char.is_delimiter_mask;
                    let delimiter_bonus =
                        delimiter_bonus_mask.select(N::DELIMITER_BONUS, N::ZERO_VEC);

                    capitalization_bonus + delimiter_bonus + N::MATCH_SCORE
                } else {
                    // Give a bonus for prefix matches
                    N::PREFIX_MATCH_SCORE
                },
            diag.saturating_sub(N::MISMATCH_PENALTY),
        );

        // Load and calculate up scores (skipping char in haystack)
        let up_gap_penalty = up_gap_penalty_mask.select(N::GAP_OPEN_PENALTY, N::GAP_EXTEND_PENALTY);
        let up_score = up_score_simd.saturating_sub(up_gap_penalty);

        // Load and calculate left scores (skipping char in needle)
        let left_gap_penalty =
            left_gap_penalty_mask.select(N::GAP_OPEN_PENALTY, N::GAP_EXTEND_PENALTY);
        let left_score = left.saturating_sub(left_gap_penalty);

        // Calculate maximum scores
        let max_score = diag_score.simd_max(up_score).simd_max(left_score);

        // Update gap penalty mask
        let diag_mask: Mask<N::Mask, L> = max_score.simd_eq(diag_score);
        up_gap_penalty_mask = max_score.simd_ne(up_score) | diag_mask;
        left_gap_penalty_mask = max_score.simd_ne(left_score) | diag_mask;

        // Only enable delimiter bonus if we've seen a non-delimiter char
        delimiter_bonus_enabled_mask |= haystack_char.is_delimiter_mask.not();

        // Store the scores for the next iterations
        up_score_simd = max_score;
        curr_score_col[haystack_idx] = max_score;

        // Store the maximum score across all runs
    }
}

#[inline]
pub fn smith_waterman<N, const W: usize, const L: usize>(
    needle: &str,
    haystacks: &[&str; L],
    max_typos: Option<u16>,
) -> ([u16; L], Vec<[Simd<N, L>; W]>, [bool; L])
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    Simd<N, L>: SimdVec<N, L>,
    Mask<N::Mask, L>: SimdMask<N, L>,
{
    let needle_str = needle;
    let needle = needle.as_bytes();

    let haystack: [HaystackChar<N, L>; W] =
        std::array::from_fn(|i| HaystackChar::from_haystacks(haystacks, i));

    // State
    let mut score_matrix = vec![[N::ZERO_VEC; W]; needle.len()];

    for (needle_idx, haystack_start, haystack_end) in (0..needle.len()).map(|needle_idx| {
        // When matching "asd" against "qwerty" with max_typos = 0, we can avoid matching "s"
        // against the "q" since it's impossible for this to be a valid match
        // And likewise, we avoid matching "d" against "q" and "w"
        let haystack_start = max_typos
            .map(|max_typos| needle_idx.saturating_sub(max_typos as usize))
            .unwrap_or(0);
        // When matching "foo" against "foobar" with max_typos = 0, we can avoid matching "f"
        // againt "a" and "r" since it's impossible for this to be a valid match
        let haystack_end = max_typos
            .map(|max_typos| (W - needle.len() + needle_idx + (max_typos as usize)).min(W))
            .unwrap_or(W);
        (needle_idx, haystack_start, haystack_end)
    }) {
        let needle_char = NeedleChar::new(N::from(needle[needle_idx]));

        let (prev_score_col, curr_score_col) = if needle_idx == 0 {
            (None, &mut score_matrix[needle_idx])
        } else {
            let (a, b) = score_matrix.split_at_mut(needle_idx);
            (Some(a[needle_idx - 1].as_slice()), &mut b[0])
        };

        smith_waterman_inner(
            haystack_start,
            haystack_end,
            needle_char,
            &haystack,
            prev_score_col,
            curr_score_col,
        );
    }

    let exact_matches = std::array::from_fn(|i| haystacks[i] == needle_str);

    let mut all_time_max_score = N::ZERO_VEC;
    for score_col in score_matrix.iter() {
        for score in score_col {
            all_time_max_score = score.simd_max(all_time_max_score);
        }
    }

    let max_scores_vec = std::array::from_fn(|i| {
        let mut score = all_time_max_score[i].into();
        if exact_matches[i] {
            score += EXACT_MATCH_BONUS;
        }
        score
    });

    (max_scores_vec, score_matrix, exact_matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAR_SCORE: u16 = MATCH_SCORE + MATCHING_CASE_BONUS;

    fn get_score(needle: &str, haystack: &str) -> u16 {
        smith_waterman::<u16, 16, 1>(needle, &[haystack; 1], None).0[0]
    }

    #[test]
    fn test_score_basic() {
        assert_eq!(get_score("b", "abc"), CHAR_SCORE);
        assert_eq!(get_score("c", "abc"), CHAR_SCORE);
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
        assert_eq!(get_score("D", "foRDist"), CHAR_SCORE);
        assert_eq!(get_score("D", "FOR_DIST"), CHAR_SCORE + DELIMITER_BONUS);
    }

    #[test]
    fn test_score_prefix_beats_delimiter() {
        assert!(get_score("swap", "swap(test)") > get_score("swap", "iter_swap(test)"));
        assert!(get_score("_", "_private_member") > get_score("_", "public_member"));
    }
}
