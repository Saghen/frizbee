use smith_waterman_macro::generate_smith_waterman;
use std::ops::{BitAnd, BitOr, Not};
use std::simd::cmp::*;
use std::simd::{Mask, Simd};

// 128-bit SIMD with u8
// NOTE: going above 16 results in much slower performance
pub const SIMD_WIDTH: usize = 16;

// NOTE: be vary carefuly when changing these values since they affect what can fit in
// the u8 scoring without overflowing
// TODO: control if we use u8 or u16 for scoring based on the scoring values and the size
// of the needle

const MATCH_SCORE: u8 = 4; // 1
const MISMATCH_PENALTY: u8 = 2; // -1,
const GAP_OPEN_PENALTY: u8 = 2; // -2
const GAP_EXTEND_PENALTY: u8 = 1; // -1

// bonus for matching the first character of the haystack
const PREFIX_BONUS: u8 = 4;
// bonus for matching character after a delimiter in the haystack (e.g. space, comma, underscore, slash, etc)
const DELIMITER_BONUS: u8 = 1;
// bonus for matching a letter that is capitalized on the haystack
const CAPITALIZATION_BONUS: u8 = 1;
// bonus multiplier for the first character of the needle
const FIRST_CHAR_MULTIPLIER: u8 = 2;

generate_smith_waterman!(4);
generate_smith_waterman!(8);
generate_smith_waterman!(12);
generate_smith_waterman!(16);
generate_smith_waterman!(24);
generate_smith_waterman!(32);
generate_smith_waterman!(48);
generate_smith_waterman!(64);
generate_smith_waterman!(96);
generate_smith_waterman!(128);
generate_smith_waterman!(160);
generate_smith_waterman!(192);
generate_smith_waterman!(224);
generate_smith_waterman!(256);
generate_smith_waterman!(384);
generate_smith_waterman!(512);

pub fn interleave_strings(strings: &[&str]) -> [[u8; SIMD_WIDTH]; 8] {
    let mut cased_result = [[0; SIMD_WIDTH]; 8];

    for (char_idx, cased_slice) in cased_result.iter_mut().enumerate() {
        for str_idx in 0..SIMD_WIDTH {
            if let Some(char) = strings[str_idx].as_bytes().get(char_idx) {
                cased_slice[str_idx] = *char;
            }
        }
    }

    cased_result
}

type SimdVec = Simd<u8, SIMD_WIDTH>;

pub fn smith_waterman_inter_simd(needle: &str, haystacks: &[&str]) -> [u16; SIMD_WIDTH] {
    let needle = needle.as_bytes();
    let needle_len = needle.len();
    let haystack_len = haystacks.iter().map(|x| x.len()).max().unwrap();

    let haystack = interleave_strings(haystacks);

    // State
    let mut prev_col_score_simds: [SimdVec; 9] = [Simd::splat(0); 9];
    let mut left_gap_penalty_masks = [Mask::splat(true); 8];
    let mut all_time_max_score = Simd::splat(0);

    // Delimiters
    let mut is_delimiter_masks = [Mask::splat(false); 8];
    let space_delimiter = Simd::splat(" ".bytes().next().unwrap() as u8);
    let slash_delimiter = Simd::splat("/".bytes().next().unwrap() as u8);
    let dot_delimiter = Simd::splat(".".bytes().next().unwrap() as u8);
    let comma_delimiter = Simd::splat(",".bytes().next().unwrap() as u8);
    let underscore_delimiter = Simd::splat("_".bytes().next().unwrap() as u8);
    let dash_delimiter = Simd::splat("-".bytes().next().unwrap() as u8);
    let delimiter_bonus = Simd::splat(DELIMITER_BONUS);

    // Capitalization
    let capital_start = Simd::splat("A".bytes().next().unwrap() as u8);
    let capital_end = Simd::splat("Z".bytes().next().unwrap() as u8);
    let capitalization_bonus = Simd::splat(CAPITALIZATION_BONUS);
    let to_lowercase_mask = Simd::splat(0x20);

    // Scoring params
    let gap_open_penalty = Simd::splat(GAP_OPEN_PENALTY);
    let gap_extend_penalty = Simd::splat(GAP_EXTEND_PENALTY);

    let match_score = Simd::splat(MATCH_SCORE);
    let mismatch_score = Simd::splat(MISMATCH_PENALTY);
    let prefix_match_score = Simd::splat(MATCH_SCORE + PREFIX_BONUS);
    let first_char_match_score = Simd::splat(MATCH_SCORE * FIRST_CHAR_MULTIPLIER);
    let first_char_prefix_match_score =
        Simd::splat((MATCH_SCORE + PREFIX_BONUS) * FIRST_CHAR_MULTIPLIER);

    let zero: SimdVec = Simd::splat(0);

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

        let needle_char = Simd::splat(needle[i - 1]);
        let mut up_score_simd = Simd::splat(0);
        let mut up_gap_penalty_mask = Mask::splat(true);
        let mut curr_col_score_simds: [SimdVec; 9] = [Simd::splat(0); 9];

        for j in 1..=haystack_len {
            let prefix_mask = Mask::splat(j == 1);
            // Load chunk and remove casing
            let cased_haystack_simd = SimdVec::from_slice(&haystack[j - 1]);
            let capital_mask = cased_haystack_simd
                .simd_ge(capital_start)
                .bitand(cased_haystack_simd.simd_le(capital_end));
            let haystack_simd = cased_haystack_simd | capital_mask.select(to_lowercase_mask, zero);

            // Give a bonus for prefix matches
            let match_score = prefix_mask.select(prefix_match_score, match_score);

            // Calculate diagonal (match/mismatch) scores
            let diag = prev_col_score_simds[j - 1];
            let match_mask = needle_char.simd_eq(haystack_simd);
            let diag_score = match_mask.select(
                diag + match_score
                    + is_delimiter_masks[j - 1].select(delimiter_bonus, zero)
                    // XOR with prefix mask to ignore capitalization on the prefix
                    + capital_mask.bitand(prefix_mask.not()).select(capitalization_bonus, zero),
                diag.simd_gt(mismatch_score)
                    .select(diag - mismatch_score, zero),
            );

            // Load and calculate up scores
            let up_gap_penalty = up_gap_penalty_mask.select(gap_open_penalty, gap_extend_penalty);
            let up_score = up_score_simd
                .simd_gt(up_gap_penalty)
                .select(up_score_simd - up_gap_penalty, zero);

            // Load and calculate left scores
            let left = prev_col_score_simds[j];
            let left_gap_penalty_mask = left_gap_penalty_masks[j - 1];
            let left_gap_penalty =
                left_gap_penalty_mask.select(gap_open_penalty, gap_extend_penalty);
            let left_score = left
                .simd_gt(left_gap_penalty)
                .select(left - left_gap_penalty, zero);

            // Calculate maximum scores
            let max_score: SimdVec = diag_score
                .simd_max(up_score)
                .simd_max(left_score)
                .simd_max(zero);

            // Update gap penalty mask
            let diag_mask = max_score.simd_eq(diag_score);
            up_gap_penalty_mask = max_score.simd_ne(up_score).bitor(diag_mask);
            left_gap_penalty_masks[j - 1] = max_score.simd_ne(left_score).bitor(diag_mask);

            // Update delimiter mask
            is_delimiter_masks[j - 1] = space_delimiter
                .simd_eq(needle_char)
                .bitor(slash_delimiter.simd_eq(needle_char))
                .bitor(dot_delimiter.simd_eq(needle_char))
                .bitor(comma_delimiter.simd_eq(needle_char))
                .bitor(underscore_delimiter.simd_eq(needle_char))
                .bitor(dash_delimiter.simd_eq(needle_char));

            // Store the scores for the next iterations
            up_score_simd = max_score;
            curr_col_score_simds[j] = max_score;

            // Store the maximum score across all runs
            all_time_max_score = all_time_max_score.simd_max(max_score);
        }

        prev_col_score_simds = curr_col_score_simds;
    }

    let mut max_scores_vec = [0; SIMD_WIDTH];
    for i in 0..SIMD_WIDTH {
        max_scores_vec[i] = all_time_max_score[i] as u16;
    }
    max_scores_vec
}

//pub fn char_indices_from_scores(
//    score_matrices: &[SimdScoreVec],
//    max_scores: &[u8; SIMD_WIDTH],
//    haystack_len: usize,
//) -> Vec<Vec<usize>> {
//    // Get the row and column indices of the maximum score
//    let max_scores = Simd::from_slice(max_scores);
//    let mut max_row = Simd::splat(0);
//    let mut max_col = Simd::splat(0);
//
//    for (col_idx, column) in score_matrices.chunks_exact(haystack_len).enumerate() {
//        let col_idx_simd = Simd::splat(col_idx as u8);
//        for (row_idx, score) in column.iter().enumerate() {
//            let row_idx_simd = Simd::splat(row_idx as u8);
//
//            let eq = score.simd_eq(max_scores);
//            max_row = eq.select(row_idx_simd, max_row);
//            max_col = eq.select(col_idx_simd, max_col);
//        }
//    }
//
//    let max_row_arr = max_row.to_array();
//    let max_col_arr = max_col.to_array();
//    let max_score_positions = max_row_arr
//        .iter()
//        .zip(max_col_arr.iter())
//        .map(|(row, col)| (*row as usize, *col as usize));
//
//    // Traceback and store the indices
//    let mut indices = vec![HashSet::new(); SIMD_WIDTH];
//    let row_stride = haystack_len + 1;
//    for (idx, (row_idx, col_idx)) in max_score_positions.enumerate() {
//        let indices = &mut indices[idx];
//        indices.insert(col_idx);
//
//        let mut last_idx = (row_idx, col_idx);
//        let mut score = score_matrices[row_idx * row_stride + col_idx][idx];
//        while score > 0 {
//            let (row_idx, col_idx) = last_idx;
//
//            // Gather up the scores for all possible paths
//            let diag = score_matrices[(row_idx - 1) * row_stride + col_idx - 1][idx];
//            let up = score_matrices[(row_idx - 1) * row_stride + col_idx][idx];
//            let left = score_matrices[row_idx * row_stride + col_idx - 1][idx];
//
//            // Choose the best path and store the index on the haystack if applicable
//            // TODO: is this logic correct? which route should we prefer?
//            score = diag.max(up).max(left);
//            if score == diag {
//                indices.insert(col_idx - 1);
//                last_idx = (row_idx - 1, col_idx - 1);
//            } else if score == up {
//                indices.insert(col_idx - 1);
//                last_idx = (row_idx, col_idx - 1);
//            } else {
//                last_idx = (row_idx - 1, col_idx);
//            }
//        }
//    }
//
//    indices
//        .iter()
//        .map(|indices| indices.iter().copied().collect())
//        .collect()
//}
