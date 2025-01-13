use crate::r#const::*;
use std::ops::Not;
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
    + std::convert::Into<u16>
{
    const ZERO: Self;
    const SIMD_WIDTH: usize;
}

pub trait SimdVec<N: SimdNum>:
    Sized
    + Copy
    + std::ops::Add<Output = Simd<N, { N::SIMD_WIDTH }>>
    + std::ops::BitOr<Output = Simd<N, { N::SIMD_WIDTH }>>
    + std::simd::cmp::SimdPartialEq<Mask = Mask<<N as SimdElement>::Mask, { N::SIMD_WIDTH }>>
    + std::simd::cmp::SimdOrd
    + std::simd::num::SimdUint
where
    N: SimdNum,
    N::Mask: std::simd::MaskElement,
    std::simd::LaneCount<{ N::SIMD_WIDTH }>: std::simd::SupportedLaneCount,
{
}

pub trait SimdMask<N: SimdNum>:
    Sized
    + Copy
    + std::ops::Not<Output = Mask<N::Mask, { N::SIMD_WIDTH }>>
    + std::ops::BitAnd<Output = Mask<N::Mask, { N::SIMD_WIDTH }>>
    + std::ops::BitOr<Output = Mask<N::Mask, { N::SIMD_WIDTH }>>
    + std::simd::cmp::SimdPartialEq<Mask = Mask<<N as SimdElement>::Mask, { N::SIMD_WIDTH }>>
where
    std::simd::LaneCount<{ N::SIMD_WIDTH }>: std::simd::SupportedLaneCount,
{
}

impl SimdNum for u8 {
    const ZERO: Self = 0;
    const SIMD_WIDTH: usize = 16;
}
impl SimdVec<u8> for Simd<u8, { <u8 as SimdNum>::SIMD_WIDTH }> {}
impl SimdMask<u8> for Mask<<u8 as SimdElement>::Mask, { <u8 as SimdNum>::SIMD_WIDTH }> {}

impl SimdNum for u16 {
    const ZERO: Self = 0;
    const SIMD_WIDTH: usize = 8;
}
impl SimdVec<u16> for Simd<u16, { <u16 as SimdNum>::SIMD_WIDTH }> {}
impl SimdMask<u16> for Mask<<u16 as SimdElement>::Mask, { <u16 as SimdNum>::SIMD_WIDTH }> {}

pub fn smith_waterman<N, const W: usize>(needle: &str, haystacks: &[&str]) -> [u16; N::SIMD_WIDTH]
where
    N: SimdNum,
    std::simd::LaneCount<{ N::SIMD_WIDTH }>: std::simd::SupportedLaneCount,
    Simd<N, { N::SIMD_WIDTH }>: SimdVec<N>,
    Mask<N::Mask, { N::SIMD_WIDTH }>: SimdMask<N>,
    [(); W + 1]: Sized,
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

    let zero = Simd::splat(N::ZERO);

    // State
    let mut score_matrix = vec![[Simd::splat(N::ZERO); W + 1]; needle.len() + 1];
    let mut left_gap_penalty_masks = [Mask::splat(true); W];
    let mut all_time_max_score = Simd::splat(N::ZERO);
    let mut all_time_max_score_row = Simd::splat(0.into());
    let mut all_time_max_score_col = Simd::splat(0.into());

    // Delimiters
    let mut delimiter_bonus_enabled_mask = Mask::splat(false);
    let mut is_delimiter_masks = [Mask::splat(false); W + 1];
    let space_delimiter = Simd::splat(N::from(b' '));
    let slash_delimiter = Simd::splat(N::from(b'/'));
    let dot_delimiter = Simd::splat(N::from(b'.'));
    let comma_delimiter = Simd::splat(N::from(b','));
    let underscore_delimiter = Simd::splat(N::from(b'_'));
    let dash_delimiter = Simd::splat(N::from(b'-'));
    let colon_delimiter = Simd::splat(N::from(b':'));
    let delimiter_bonus = Simd::splat(N::from(DELIMITER_BONUS));

    // Capitalization
    let capital_start = Simd::splat(N::from(b'A'));
    let capital_end = Simd::splat(N::from(b'Z'));
    let capitalization_bonus = Simd::splat(N::from(CAPITALIZATION_BONUS));
    let matching_casing_bonus = Simd::splat(N::from(MATCHING_CASE_BONUS));
    let to_lowercase_mask = Simd::splat(N::from(0x20));

    // Scoring params
    let gap_open_penalty = Simd::splat(N::from(GAP_OPEN_PENALTY));
    let gap_extend_penalty = Simd::splat(N::from(GAP_EXTEND_PENALTY));

    let match_score = Simd::splat(N::from(MATCH_SCORE));
    let mismatch_score = Simd::splat(N::from(MISMATCH_PENALTY));
    let prefix_match_score = Simd::splat(N::from(MATCH_SCORE + PREFIX_BONUS));

    for i in 1..=needle_len {
        let prev_col_scores = score_matrix[i - 1];
        let curr_col_scores = &mut score_matrix[i];

        let mut up_score_simd = Simd::splat(N::ZERO);
        let mut up_gap_penalty_mask = Mask::splat(true);

        let needle_char = Simd::splat(needle[i - 1]);
        let needle_cased_mask: Mask<N::Mask, { N::SIMD_WIDTH }> =
            needle_char.simd_ge(capital_start) & needle_char.simd_le(capital_end);
        let needle_char = needle_char | needle_cased_mask.select(to_lowercase_mask, zero);

        for j in 1..=haystack_len {
            let prefix_mask = Mask::splat(j == 1);

            // Load chunk and remove casing
            let cased_haystack_simd = Simd::from_slice(&haystack[j - 1]);
            let capital_mask: Mask<N::Mask, { N::SIMD_WIDTH }> = cased_haystack_simd
                .simd_ge(capital_start)
                & cased_haystack_simd.simd_le(capital_end);
            let haystack_simd = cased_haystack_simd | capital_mask.select(to_lowercase_mask, zero);

            let matched_casing_mask = needle_cased_mask.simd_eq(capital_mask);

            // Give a bonus for prefix matches
            let match_score = prefix_mask.select(prefix_match_score, match_score);

            // Calculate diagonal (match/mismatch) scores
            let diag = prev_col_scores[j - 1];
            let match_mask: Mask<N::Mask, { N::SIMD_WIDTH }> = needle_char.simd_eq(haystack_simd);
            let diag_score: Simd<N, { N::SIMD_WIDTH }> = match_mask.select(
                diag + match_score
                    + (is_delimiter_masks[j - 1] & delimiter_bonus_enabled_mask).select(delimiter_bonus, zero)
                    // XOR with prefix mask to ignore capitalization on the prefix
                    + (capital_mask & prefix_mask.not()).select(capitalization_bonus, zero)
                    + matched_casing_mask.select(matching_casing_bonus, zero),
                diag.saturating_sub(mismatch_score),
            );

            // Load and calculate up scores (skipping char in haystack)
            let up_gap_penalty = up_gap_penalty_mask.select(gap_open_penalty, gap_extend_penalty);
            let up_score = up_score_simd.saturating_sub(up_gap_penalty);

            // Load and calculate left scores (skipping char in needle)
            let left = prev_col_scores[j];
            let left_gap_penalty_mask = left_gap_penalty_masks[j - 1];
            let left_gap_penalty =
                left_gap_penalty_mask.select(gap_open_penalty, gap_extend_penalty);
            let left_score = left.saturating_sub(left_gap_penalty);

            // Calculate maximum scores
            let max_score = diag_score.simd_max(up_score).simd_max(left_score);

            // Update gap penalty mask
            let diag_mask: Mask<N::Mask, { N::SIMD_WIDTH }> = max_score.simd_eq(diag_score);
            up_gap_penalty_mask = max_score.simd_ne(up_score) | diag_mask;
            left_gap_penalty_masks[j - 1] = max_score.simd_ne(left_score) | diag_mask;

            // Update delimiter masks
            is_delimiter_masks[j] = space_delimiter.simd_eq(haystack_simd)
                | slash_delimiter.simd_eq(haystack_simd)
                | dot_delimiter.simd_eq(haystack_simd)
                | comma_delimiter.simd_eq(haystack_simd)
                | underscore_delimiter.simd_eq(haystack_simd)
                | dash_delimiter.simd_eq(haystack_simd)
                | colon_delimiter.simd_eq(haystack_simd);
            // Only enable delimiter bonus if we've seen a non-delimiter char
            delimiter_bonus_enabled_mask |= is_delimiter_masks[j].not();

            // Store the scores for the next iterations
            up_score_simd = max_score;
            curr_col_scores[j] = max_score;

            // Store the maximum score across all runs
            // TODO: shouldn't we only care about the max score of the final column?
            // since we want to match the entire needle to see how many typos there are
            let all_time_max_score_mask: Mask<N::Mask, { N::SIMD_WIDTH }> =
                all_time_max_score.simd_lt(max_score);
            // TODO: must guarantee that needle.len() < 2 ** (8 || 16)
            all_time_max_score_col = all_time_max_score_mask.select(
                Simd::splat(N::from(i.try_into().unwrap())),
                all_time_max_score_col,
            );
            all_time_max_score_row = all_time_max_score_mask.select(
                Simd::splat(N::from(j.try_into().unwrap())),
                all_time_max_score_row,
            );
            all_time_max_score = all_time_max_score.simd_max(max_score);
        }
    }

    let mut max_scores_vec = [0u16; N::SIMD_WIDTH];
    for i in 0..N::SIMD_WIDTH {
        max_scores_vec[i] = all_time_max_score[i].into();
        if haystacks[i] == needle_str {
            max_scores_vec[i] += EXACT_MATCH_BONUS as u16;
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
        smith_waterman::<u8, SIMD_WIDTH>(needle, &haystacks)[0] as u8
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
