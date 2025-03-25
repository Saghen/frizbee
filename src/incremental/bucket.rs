use std::simd::{cmp::SimdOrd, Simd};

use crate::{
    smith_waterman::simd::{
        smith_waterman_inner, typos_from_score_matrix, HaystackChar, NeedleChar, SimdMask, SimdNum,
        SimdVec,
    },
    Match,
};

pub(crate) trait IncrementalBucketTrait {
    fn process(
        &mut self,
        prefix_to_keep: usize,
        new_needle_chars: &[u8],
        matches: &mut Vec<Match>,
        min_score: u16,
        max_typos: Option<u16>,
    );
}

pub(crate) struct IncrementalBucket<N: SimdNum<L>, const W: usize, const L: usize>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    pub length: usize,
    pub idxs: [usize; L],
    pub haystacks: [HaystackChar<N, L>; W],
    pub score_matrix: Vec<[Simd<N, L>; W]>,
}

impl<N: SimdNum<L>, const W: usize, const L: usize> IncrementalBucket<N, W, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    std::simd::Simd<N, L>: SimdVec<N, L>,
    std::simd::Mask<N::Mask, L>: SimdMask<N, L>,
{
    pub fn new(haystacks: &[&str; L], idxs: [usize; L], length: usize) -> Self {
        Self {
            length,
            idxs,
            haystacks: std::array::from_fn(|i| HaystackChar::from_haystacks(haystacks, i)),
            score_matrix: vec![],
        }
    }
}

impl<N: SimdNum<L>, const W: usize, const L: usize> IncrementalBucketTrait
    for IncrementalBucket<N, W, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    std::simd::Simd<N, L>: SimdVec<N, L>,
    std::simd::Mask<N::Mask, L>: SimdMask<N, L>,
{
    #[inline]
    fn process(
        &mut self,
        prefix_to_keep: usize,
        new_needle_chars: &[u8],
        matches: &mut Vec<Match>,
        min_score: u16,
        max_typos: Option<u16>,
    ) {
        let new_needle_chars = new_needle_chars
            .into_iter()
            .map(|&x| NeedleChar::new(x.into()))
            .collect::<Box<[_]>>();

        // Adjust score matrix to the new size
        if new_needle_chars.len() > prefix_to_keep {
            self.score_matrix.extend(std::iter::repeat_n(
                [N::ZERO_VEC; W],
                new_needle_chars.len() - prefix_to_keep,
            ));
        } else if new_needle_chars.len() < prefix_to_keep {
            self.score_matrix
                .truncate(prefix_to_keep + new_needle_chars.len());
        }

        for (i, &needle_char) in new_needle_chars.iter().enumerate() {
            let needle_idx = i + prefix_to_keep;

            let (prev_score_col, curr_score_col) = if needle_idx == 0 {
                (None, self.score_matrix[needle_idx].as_mut())
            } else {
                let (a, b) = self.score_matrix.split_at_mut(needle_idx);
                (Some(a[needle_idx - 1].as_ref()), b[0].as_mut())
            };

            smith_waterman_inner(
                W,
                needle_char,
                &self.haystacks,
                prev_score_col,
                curr_score_col,
            );
        }

        let mut all_time_max_score = N::ZERO_VEC;
        for score_col in self.score_matrix.iter() {
            for score in score_col {
                all_time_max_score = score.simd_max(all_time_max_score);
            }
        }

        // TODO: DRY w/ smith_waterman
        let scores: [u16; L] = std::array::from_fn(|i| {
            let score = all_time_max_score[i].into();
            // TODO: exact match bonus - this is going to be tricky becayse raw haystacks aren't
            // currently stored. perhaps simd the comparison?
            // if haystacks[i] == needle_str {
            //     score += EXACT_MATCH_BONUS;
            // }
            score
        });

        // TODO: typos
        let typos = max_typos.map(|_| typos_from_score_matrix::<N, W, L>(&self.score_matrix));

        #[allow(clippy::needless_range_loop)]
        for idx in 0..self.length {
            let score = scores[idx];
            if score < min_score {
                continue;
            }

            if let Some(max_typos) = max_typos {
                if typos.is_some_and(|typos| typos[idx] > max_typos) {
                    continue;
                }
            }

            let score_idx = self.idxs[idx];
            matches.push(Match {
                index_in_haystack: score_idx,
                score: scores[idx],
                indices: None,
                exact: false,
            });
        }
    }
}
