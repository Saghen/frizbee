use std::marker::PhantomData;

use crate::simd::{smith_waterman, SimdNum};
use crate::Match;

pub(crate) trait Bucket<'a> {
    fn add_haystack(&mut self, haystack: &'a str, idx: usize);
    fn is_full(&self) -> bool;
    fn process(&mut self, matches: &mut [Option<Match>], needle: &str, _with_indices: bool);
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
    std::simd::Simd<N, L>: crate::simd::SimdVec<N, L>,
    std::simd::Mask<N::Mask, L>: crate::simd::SimdMask<N, L>,
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
    std::simd::Simd<N, L>: crate::simd::SimdVec<N, L>,
    std::simd::Mask<N::Mask, L>: crate::simd::SimdMask<N, L>,
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

    fn process(&mut self, matches: &mut [Option<Match>], needle: &str, _with_indices: bool) {
        if self.length == 0 {
            return;
        }

        let scores: &[u16] = &smith_waterman::<N, W, L>(needle, &self.haystacks).0;
        #[allow(clippy::needless_range_loop)]
        for idx in 0..self.length {
            let score_idx = self.idxs[idx];
            matches[score_idx] = Some(Match {
                index_in_haystack: score_idx,
                index: score_idx,
                score: scores[idx],
                indices: None, //indices: bucket_indices.get(idx).cloned(),
            });
        }

        self.reset();
    }

    fn reset(&mut self) {
        self.length = 0;
    }
}
