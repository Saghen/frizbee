use std::collections::HashSet;
use std::simd::cmp::*;
use std::simd::{Mask, Simd};

use super::{SimdMask, SimdNum, SimdVec};

#[inline]
pub fn char_indices_from_scores<N, const W: usize, const L: usize>(
    score_matrices: &[[Simd<N, L>; W]],
) -> Vec<Vec<usize>>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    Simd<N, L>: SimdVec<N, L>,
    Mask<N::Mask, L>: SimdMask<N, L>,
{
    // Find the maximum score row/col for each haystack
    let mut max_scores = N::ZERO_VEC;
    let mut max_rows = N::ZERO_VEC;
    let mut max_cols = N::ZERO_VEC;

    for (col, col_scores) in score_matrices.iter().enumerate() {
        for (row, row_scores) in col_scores.iter().enumerate() {
            let scores_mask = row_scores.simd_ge(max_scores);

            max_rows = scores_mask.select(Simd::splat(N::from_usize(row)), max_rows);
            max_cols = scores_mask.select(Simd::splat(N::from_usize(col)), max_cols);

            max_scores = max_scores.simd_max(*row_scores);
        }
    }

    let max_score_positions = max_rows.to_array().into_iter().zip(max_cols.to_array());

    // Traceback and store the matched indices
    let mut indices = vec![HashSet::new(); L];

    for (idx, (row_idx, col_idx)) in max_score_positions.enumerate() {
        let indices = &mut indices[idx];

        let mut row_idx: usize = row_idx.into();
        let mut col_idx: usize = col_idx.into();
        let mut score = score_matrices[col_idx][row_idx][idx];

        // NOTE: row_idx = 0 or col_idx = 0 will always have a score of 0
        while score > 0.into() {
            // Gather up the scores for all possible paths
            let diag = if col_idx == 0 || row_idx == 0 {
                N::ZERO
            } else {
                score_matrices[col_idx - 1][row_idx - 1][idx]
            };
            let left = if col_idx == 0 {
                N::ZERO
            } else {
                score_matrices[col_idx - 1][row_idx][idx]
            };
            let up = if row_idx == 0 {
                N::ZERO
            } else {
                score_matrices[col_idx][row_idx - 1][idx]
            };

            // Diagonal (match/mismatch)
            if diag >= left && diag >= up {
                // Check if the score decreases (remember we're going backwards)
                // to see if we've found a match
                if diag < score {
                    indices.insert(row_idx);
                }

                row_idx = row_idx.saturating_sub(1);
                col_idx = col_idx.saturating_sub(1);

                score = diag;
            }
            // Up (gap in haystack)
            else if up >= left {
                // Finished crossing a gap, remove any previous rows
                if up > score && up > 0.into() {
                    indices.remove(&(row_idx));
                    indices.insert(row_idx.saturating_sub(1));
                }

                row_idx = row_idx.saturating_sub(1);

                score = up;
            }
            // Left (gap in needle)
            else {
                col_idx = col_idx.saturating_sub(1);
                score = left;
            }
        }
    }

    indices
        .iter()
        .map(|indices| {
            let mut indices = indices.iter().copied().collect::<Vec<_>>();
            indices.sort();
            indices
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::smith_waterman::simd::smith_waterman;

    use super::*;

    fn run_single_indices(needle: &str, haystack: &str) -> Vec<usize> {
        let haystacks = [haystack; 1];
        let (_, score_matrices, _) = smith_waterman::<u16, 16, 1>(needle, &haystacks);
        let indices = char_indices_from_scores(&score_matrices);
        indices[0].clone()
    }

    #[test]
    fn test_leaking() {
        let needle = "t";
        let haystacks = [
            "true",
            "toDate",
            "toString",
            "transpose",
            "testing",
            "to",
            "toRgba",
            "toolbar",
            "true",
            "toDate",
            "toString",
            "transpose",
            "testing",
            "to",
            "toRgba",
            "toolbar",
        ];

        let (_, score_matrices, _) = smith_waterman::<u16, 16, 16>(needle, &haystacks);
        let indices = char_indices_from_scores(&score_matrices);
        for indices in indices.into_iter() {
            assert_eq!(indices, [0])
        }
    }

    #[test]
    fn test_basic_indices() {
        assert_eq!(run_single_indices("b", "abc"), vec![1]);
        assert_eq!(run_single_indices("c", "abc"), vec![2]);
    }

    #[test]
    fn test_prefix_indices() {
        assert_eq!(run_single_indices("a", "abc"), vec![0]);
        assert_eq!(run_single_indices("a", "aabc"), vec![0]);
        assert_eq!(run_single_indices("a", "babc"), vec![1]);
    }

    #[test]
    fn test_exact_match_indices() {
        assert_eq!(run_single_indices("a", "a"), vec![0]);
        assert_eq!(run_single_indices("abc", "abc"), vec![0, 1, 2]);
        assert_eq!(run_single_indices("ab", "abc"), vec![0, 1]);
    }

    #[test]
    fn test_delimiter_indices() {
        assert_eq!(run_single_indices("b", "a-b"), vec![2]);
        assert_eq!(run_single_indices("a", "a-b-c"), vec![0]);
        assert_eq!(run_single_indices("b", "a--b"), vec![3]);
        assert_eq!(run_single_indices("c", "a--bc"), vec![4]);
    }

    #[test]
    fn test_affine_gap_indices() {
        assert_eq!(run_single_indices("test", "Uterst"), vec![1, 2, 4, 5]);
        assert_eq!(run_single_indices("test", "Uterrst"), vec![1, 2, 5, 6]);
        assert_eq!(run_single_indices("test", "Uterrs t"), vec![1, 2, 5, 7]);
    }

    #[test]
    fn test_capital_indices() {
        assert_eq!(run_single_indices("a", "A"), vec![0]);
        assert_eq!(run_single_indices("A", "Aa"), vec![0]);
        assert_eq!(run_single_indices("D", "forDist"), vec![3]);
    }

    #[test]
    fn test_typo_indices() {
        assert_eq!(run_single_indices("b", "a"), vec![]);
        assert_eq!(run_single_indices("reba", "repack"), vec![0, 1, 3]);
        assert_eq!(run_single_indices("bbb", "abc"), vec![1]);
    }
}
