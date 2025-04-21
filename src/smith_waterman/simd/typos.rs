use std::simd::cmp::*;
use std::simd::{Mask, Simd};

use super::{SimdMask, SimdNum, SimdVec};

#[inline]
pub fn typos_from_score_matrix<N, const W: usize, const L: usize>(
    score_matrix: &[[Simd<N, L>; W]],
    max_typos: u16,
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
    for (idx, &row_scores) in last_column.iter().enumerate() {
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
            if typo_count[idx] > max_typos {
                break;
            }

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
    use crate::smith_waterman::simd::smith_waterman;

    fn get_typos(needle: &str, haystack: &str) -> u16 {
        typos_from_score_matrix(
            &smith_waterman::<u16, 4, 1>(needle, &[haystack; 1], Some(1)).1,
            100,
        )[0]
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
}
