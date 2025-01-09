use crate::r#const::*;
use std::ops::{BitAnd, BitOr, Not};
use std::simd::num::SimdUint;
use std::simd::{cmp::*, SimdElement};
use std::simd::{Mask, Simd};

pub trait SimdNum:
    Sized
    + Copy
    + std::simd::SimdElement
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::convert::From<u8>
{
    const ZERO: Self;
    const SIMD_WIDTH: usize;
}
type SimdVec<N: SimdNum> = Simd<N, { N::SIMD_WIDTH }>;
type SimdMask<N: SimdNum> = Mask<<N as SimdElement>::Mask, { N::SIMD_WIDTH }>;

impl SimdNum for u8 {
    const ZERO: Self = 0;
    const SIMD_WIDTH: usize = 16;
}
impl SimdNum for u16 {
    const ZERO: Self = 0;
    const SIMD_WIDTH: usize = 8;
}

pub fn smith_waterman<N, const W: usize>(needle: &str, haystacks: &[&str]) -> [N; N::SIMD_WIDTH]
where
    N: SimdNum,
    std::simd::LaneCount<{ N::SIMD_WIDTH }>: std::simd::SupportedLaneCount,
    [(); W + 1]:,
    SimdVec<N>: std::ops::Add<Output = SimdVec<N>>
        + std::ops::BitOr<Output = SimdVec<N>>
        + std::simd::cmp::SimdPartialEq<Mask = SimdMask<N>>
        + std::simd::cmp::SimdOrd<Mask = SimdMask<N>>
        + std::simd::num::SimdUint,
    SimdMask<N>: std::ops::BitAnd<Output = SimdMask<N>>
        + std::ops::BitOr<Output = SimdMask<N>>
        + std::simd::cmp::SimdPartialEq<Mask = SimdMask<N>>,
{
    let needle_str = needle;
    let needle: Vec<N> = needle.as_bytes().iter().map(|x| N::from(*x)).collect();
    let needle_len = needle.len();
    let haystack_len = haystacks.iter().map(|x| x.len()).max().unwrap();

    // Convert haystacks to a static array of bytes chunked for SIMD
    let mut haystack: [[N; N::SIMD_WIDTH]; W] = [[N::ZERO; N::SIMD_WIDTH]; W];
    for (char_idx, haystack_slice) in haystack.iter_mut().enumerate() {
        for str_idx in 0..N::SIMD_WIDTH {
            if let Some(char) = haystacks[str_idx].as_bytes().get(char_idx) {
                haystack_slice[str_idx] = N::from(*char);
            }
        }
    }

    let zero = SimdVec::<N>::splat(N::ZERO);

    // State
    let mut prev_col_score_simds = [SimdVec::<N>::splat(N::ZERO); W + 1];
    let mut left_gap_penalty_masks = [SimdMask::<N>::splat(true); W];
    let mut all_time_max_score = SimdVec::<N>::splat(N::ZERO);

    // Delimiters
    let mut delimiter_bonus_enabled_mask = SimdMask::<N>::splat(false);
    let mut is_delimiter_masks = [SimdMask::<N>::splat(false); W + 1];
    let space_delimiter = SimdVec::<N>::splat(N::from(b' '));
    let slash_delimiter = SimdVec::<N>::splat(N::from(b'/'));
    let dot_delimiter = SimdVec::<N>::splat(N::from(b'.'));
    let comma_delimiter = SimdVec::<N>::splat(N::from(b','));
    let underscore_delimiter = SimdVec::<N>::splat(N::from(b'_'));
    let dash_delimiter = SimdVec::<N>::splat(N::from(b'-'));
    let colon_delimiter = SimdVec::<N>::splat(N::from(b':'));
    let delimiter_bonus = SimdVec::<N>::splat(N::from(DELIMITER_BONUS));

    // Capitalization
    let capital_start = SimdVec::<N>::splat(N::from(b'A'));
    let capital_end = SimdVec::<N>::splat(N::from(b'Z'));
    let capitalization_bonus = SimdVec::<N>::splat(N::from(CAPITALIZATION_BONUS));
    let matching_casing_bonus = SimdVec::<N>::splat(N::from(MATCHING_CASE_BONUS));
    let to_lowercase_mask = SimdVec::<N>::splat(N::from(0x20));

    // Scoring params
    let gap_open_penalty = SimdVec::<N>::splat(N::from(GAP_OPEN_PENALTY));
    let gap_extend_penalty = SimdVec::<N>::splat(N::from(GAP_EXTEND_PENALTY));

    let match_score = SimdVec::<N>::splat(N::from(MATCH_SCORE));
    let mismatch_score = SimdVec::<N>::splat(N::from(MISMATCH_PENALTY));
    let prefix_match_score = SimdVec::<N>::splat(N::from(MATCH_SCORE + PREFIX_BONUS));
    let first_char_match_score = SimdVec::<N>::splat(N::from(MATCH_SCORE * FIRST_CHAR_MULTIPLIER));
    let first_char_prefix_match_score = SimdVec::<N>::splat(N::from(
        (MATCH_SCORE + PREFIX_BONUS) * FIRST_CHAR_MULTIPLIER,
    ));

    for i in 1..=needle_len {
        let match_score = if i == 1 {
            first_char_match_score
        } else {
            match_score
        };
        let prefix_match_score = if i == 1 {
            first_char_prefix_match_score
        } else {
            prefix_match_score
        };

        let needle_char = SimdVec::<N>::splat(needle[i - 1]);
        let mut up_score_simd = SimdVec::splat(N::ZERO);
        let mut up_gap_penalty_mask = SimdMask::<N>::splat(true);
        let mut curr_col_score_simds: [SimdVec<N>; W + 1] = [SimdVec::<N>::splat(N::ZERO); W + 1];
        let needle_cased_mask = needle_char
            .simd_ge(capital_start)
            .bitand(needle_char.simd_le(capital_end));
        let needle_char = needle_char | needle_cased_mask.select(to_lowercase_mask, zero);

        for j in 1..=haystack_len {
            let prefix_mask = SimdMask::<N>::splat(j == 1);

            // Load chunk and remove casing
            let cased_haystack_simd = SimdVec::<N>::from_slice(&haystack[j - 1]);
            let capital_mask = cased_haystack_simd
                .simd_ge(capital_start)
                .bitand(cased_haystack_simd.simd_le(capital_end));
            let haystack_simd = cased_haystack_simd | capital_mask.select(to_lowercase_mask, zero);

            let matched_casing_mask = needle_cased_mask.simd_eq(capital_mask);

            // Give a bonus for prefix matches
            let match_score = prefix_mask.select(prefix_match_score, match_score);

            // Calculate diagonal (match/mismatch) scores
            let diag = prev_col_score_simds[j - 1];
            let match_mask = needle_char.simd_eq(haystack_simd);
            let diag_score = match_mask.select(
                diag + match_score
                    + is_delimiter_masks[j - 1].bitand(delimiter_bonus_enabled_mask).select(delimiter_bonus, zero)
                    // XOR with prefix mask to ignore capitalization on the prefix
                    + capital_mask.bitand(prefix_mask.not()).select(capitalization_bonus, zero)
                    + matched_casing_mask.select(matching_casing_bonus, zero),
                diag.saturating_sub(mismatch_score),
            );

            // Load and calculate up scores
            let up_gap_penalty = up_gap_penalty_mask.select(gap_open_penalty, gap_extend_penalty);
            let up_score = up_score_simd.saturating_sub(up_gap_penalty);

            // Load and calculate left scores
            let left = prev_col_score_simds[j];
            let left_gap_penalty_mask = left_gap_penalty_masks[j - 1];
            let left_gap_penalty =
                left_gap_penalty_mask.select(gap_open_penalty, gap_extend_penalty);
            let left_score = left.saturating_sub(left_gap_penalty);

            // Calculate maximum scores
            let max_score = diag_score.simd_max(up_score).simd_max(left_score);

            // Update gap penalty mask
            let diag_mask = max_score.simd_eq(diag_score);
            up_gap_penalty_mask = max_score.simd_ne(up_score).bitor(diag_mask);
            left_gap_penalty_masks[j - 1] = max_score.simd_ne(left_score).bitor(diag_mask);

            // Update delimiter masks
            is_delimiter_masks[j] = space_delimiter
                .simd_eq(haystack_simd)
                .bitor(slash_delimiter.simd_eq(haystack_simd))
                .bitor(dot_delimiter.simd_eq(haystack_simd))
                .bitor(comma_delimiter.simd_eq(haystack_simd))
                .bitor(underscore_delimiter.simd_eq(haystack_simd))
                .bitor(dash_delimiter.simd_eq(haystack_simd))
                .bitor(colon_delimiter.simd_eq(haystack_simd));
            // Only enable delimiter bonus if we've seen a non-delimiter char
            delimiter_bonus_enabled_mask =
                delimiter_bonus_enabled_mask.bitor(is_delimiter_masks[j].not());

            // Store the scores for the next iterations
            up_score_simd = max_score;
            curr_col_score_simds[j] = max_score;

            // Store the maximum score across all runs
            all_time_max_score = all_time_max_score.simd_max(max_score);
        }

        prev_col_score_simds = curr_col_score_simds;
    }

    let mut max_scores_vec = [N::ZERO; N::SIMD_WIDTH];
    for i in 0..N::SIMD_WIDTH {
        max_scores_vec[i] = all_time_max_score[i];
        if haystacks[i] == needle_str {
            max_scores_vec[i] += N::from(EXACT_MATCH_BONUS);
        }
    }
    max_scores_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAR_SCORE: u8 = MATCH_SCORE + MATCHING_CASE_BONUS;

    fn run_single(needle: &str, haystack: &str) -> u8 {
        let haystacks = [haystack; SIMD_WIDTH];
        smith_waterman::<u8, SIMD_WIDTH>(needle, &haystacks)[0]
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
            CHAR_SCORE + EXACT_MATCH_BONUS + PREFIX_BONUS
        );
        assert_eq!(
            run_single("abc", "abc"),
            3 * CHAR_SCORE + EXACT_MATCH_BONUS + PREFIX_BONUS
        );
        assert_eq!(run_single("ab", "abc"), 2 * CHAR_SCORE + PREFIX_BONUS);
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
