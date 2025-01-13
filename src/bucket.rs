use crate::simd::{smith_waterman, SimdNum};
use crate::Match;

pub(crate) trait Bucket<'a> {
    fn add_haystack(&mut self, haystack: &'a str, idx: usize);
    fn is_full(&self) -> bool;
    fn process(&mut self, matches: &mut [Option<Match>], needle: &str, _with_indices: bool);
    fn reset(&mut self);
}

pub(crate) struct FixedWidthBucket<'a, N: SimdNum, const W: usize>
where
    [(); N::SIMD_WIDTH]: Sized,
{
    length: usize,
    haystacks: [&'a str; N::SIMD_WIDTH],
    idxs: [usize; N::SIMD_WIDTH],
}

impl<N: SimdNum, const W: usize> FixedWidthBucket<'_, N, W>
where
    [(); N::SIMD_WIDTH]: Sized,
{
    pub fn new() -> Self {
        FixedWidthBucket {
            length: 0,
            haystacks: [""; N::SIMD_WIDTH],
            idxs: [0; N::SIMD_WIDTH],
        }
    }
}

impl<'a, N: SimdNum, const W: usize> Bucket<'a> for FixedWidthBucket<'a, N, W>
where
    [(); W + 1]: Sized,
    std::simd::LaneCount<{ N::SIMD_WIDTH }>: std::simd::SupportedLaneCount,
    std::simd::Simd<N, { N::SIMD_WIDTH }>: crate::simd::SimdVec<N>,
    std::simd::Mask<N::Mask, { N::SIMD_WIDTH }>: crate::simd::SimdMask<N>,
{
    fn add_haystack(&mut self, haystack: &'a str, idx: usize) {
        assert!(haystack.len() <= W);
        if self.length == N::SIMD_WIDTH {
            return;
        }
        self.haystacks[self.length] = haystack;
        self.idxs[self.length] = idx;
        self.length += 1;
    }

    fn is_full(&self) -> bool {
        self.length == N::SIMD_WIDTH
    }

    fn process(&mut self, matches: &mut [Option<Match>], needle: &str, _with_indices: bool) {
        if self.length == 0 {
            return;
        }

        let scores: &[u16] = &smith_waterman::<N, W>(needle, &self.haystacks);
        for idx in 0..self.length {
            let score_idx = self.idxs[idx];
            matches[score_idx] = Some(Match {
                index_in_haystack: score_idx,
                index: score_idx,
                score: scores[idx] as u16,
                indices: None, //indices: bucket_indices.get(idx).cloned(),
            });
        }

        self.reset();
    }

    fn reset(&mut self) {
        self.length = 0;
    }
}
