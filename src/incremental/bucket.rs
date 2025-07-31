use std::simd::{cmp::SimdOrd, Simd};

use crate::{
    smith_waterman::simd::{
        smith_waterman_inner, typos_from_score_matrix, HaystackChar, NeedleChar,
    },
    Match,
};

pub(crate) trait IncrementalBucketTrait {
    fn process(
        &mut self,
        prefix_to_keep: usize,
        new_needle_chars: &[u8],
        matches: &mut Vec<Match>,
        max_typos: Option<u16>,
    );
}

pub(crate) struct IncrementalBucket<const W: usize, const L: usize>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    pub length: usize,
    pub idxs: [u32; L],
    pub haystacks: [HaystackChar<L>; W],
    pub score_matrix: Vec<[Simd<u16, L>; W]>,
}

impl<const W: usize, const L: usize> IncrementalBucket<W, L>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    pub fn new(haystacks: &[&str; L], idxs: [u32; L], length: usize) -> Self {
        Self {
            length,
            idxs,
            haystacks: std::array::from_fn(|i| HaystackChar::from_haystack(haystacks, i)),
            score_matrix: vec![],
        }
    }
}

impl<const W: usize, const L: usize> IncrementalBucketTrait for IncrementalBucket<W, L>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    #[inline]
    fn process(
        &mut self,
        prefix_to_keep: usize,
        new_needle_chars: &[u8],
        matches: &mut Vec<Match>,
        max_typos: Option<u16>,
    ) {
        let new_needle_chars = new_needle_chars
            .iter()
            .map(|&x| NeedleChar::new(x.into()))
            .collect::<Box<[_]>>();

        // Adjust score matrix to the new size
        if new_needle_chars.len() > prefix_to_keep {
            self.score_matrix.extend(std::iter::repeat_n(
                [Simd::splat(0); W],
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
                0,
                W,
                needle_char,
                &self.haystacks,
                prev_score_col,
                curr_score_col,
            );
        }

        let mut all_time_max_score = Simd::splat(0);
        for score_col in self.score_matrix.iter() {
            for score in score_col {
                all_time_max_score = score.simd_max(all_time_max_score);
            }
        }

        // TODO: DRY w/ smith_waterman
        let scores: [u16; L] = std::array::from_fn(|i| {
            all_time_max_score[i].into()
            // TODO: exact match bonus - this is going to be tricky because raw haystacks aren't
            // currently stored. perhaps simd the comparison?
            // if haystacks[i] == needle_str {
            //     score += EXACT_MATCH_BONUS;
            // }
        });

        // TODO: typos
        let typos = max_typos
            .map(|max_typos| typos_from_score_matrix::<W, L>(&self.score_matrix, max_typos));

        #[allow(clippy::needless_range_loop)]
        for idx in 0..self.length {
            if let Some(max_typos) = max_typos {
                if typos.is_some_and(|typos| typos[idx] > max_typos) {
                    continue;
                }
            }

            let score_idx = self.idxs[idx];
            matches.push(Match {
                index_in_haystack: score_idx,
                score: scores[idx],
                exact: false,
            });
        }
    }
}
