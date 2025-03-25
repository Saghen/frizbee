use std::marker::PhantomData;

use crate::smith_waterman::simd::{
    smith_waterman, typos_from_score_matrix, SimdMask, SimdNum, SimdVec,
};
use crate::Match;

pub(crate) trait Bucket<'a> {
    fn add_haystack(&mut self, haystack: &'a str, idx: usize);
    fn is_full(&self) -> bool;
    fn process(
        &mut self,
        matches: &mut Vec<Match>,
        needle: &str,
        min_score: u16,
        max_typos: Option<u16>,
    );
    fn reset(&mut self);
}

pub(crate) struct FixedWidthBucket<'a, N: SimdNum<L>, const W: usize, const L: usize>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    length: usize,
    haystacks: [&'a str; L],
    idxs: [usize; L],
    _phantom: PhantomData<N>,
}

impl<N: SimdNum<L>, const W: usize, const L: usize> FixedWidthBucket<'_, N, W, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    std::simd::Simd<N, L>: SimdVec<N, L>,
    std::simd::Mask<N::Mask, L>: SimdMask<N, L>,
{
    pub fn new() -> Self {
        FixedWidthBucket {
            length: 0,
            haystacks: [""; L],
            idxs: [0; L],
            _phantom: PhantomData,
        }
    }
}

impl<'a, N: SimdNum<L>, const W: usize, const L: usize> Bucket<'a> for FixedWidthBucket<'a, N, W, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    std::simd::Simd<N, L>: SimdVec<N, L>,
    std::simd::Mask<N::Mask, L>: SimdMask<N, L>,
{
    fn add_haystack(&mut self, haystack: &'a str, idx: usize) {
        assert!(haystack.len() <= W);
        if self.length == L {
            return;
        }
        self.haystacks[self.length] = haystack;
        self.idxs[self.length] = idx;
        self.length += 1;
    }

    fn is_full(&self) -> bool {
        self.length == L
    }

    fn process(
        &mut self,
        matches: &mut Vec<Match>,
        needle: &str,
        min_score: u16,
        max_typos: Option<u16>,
    ) {
        if self.length == 0 {
            return;
        }

        let (scores, score_matrix, exact_matches) =
            smith_waterman::<N, W, L>(needle, &self.haystacks);

        let typos = max_typos.map(|_| typos_from_score_matrix::<N, W, L>(&score_matrix));
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
                exact: exact_matches[idx],
                indices: None,
            });
        }

        self.reset();
    }

    fn reset(&mut self) {
        self.length = 0;
    }
}
