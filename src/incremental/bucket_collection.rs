use std::marker::PhantomData;

use crate::smith_waterman::simd::{SimdMask, SimdNum, SimdVec};

use super::bucket::{IncrementalBucket, IncrementalBucketTrait};

pub(crate) struct IncrementalBucketCollection<'a, N: SimdNum<L>, const W: usize, const L: usize>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    length: usize,
    haystacks: [&'a str; L],
    idxs: [usize; L],
    _phantom: PhantomData<N>,
}

impl<'a, N: SimdNum<L> + 'static, const W: usize, const L: usize>
    IncrementalBucketCollection<'a, N, W, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    std::simd::Simd<N, L>: SimdVec<N, L>,
    std::simd::Mask<N::Mask, L>: SimdMask<N, L>,
{
    pub fn new() -> Self {
        Self {
            length: 0,
            haystacks: [""; L],
            idxs: [0; L],
            _phantom: PhantomData,
        }
    }

    fn build_bucket(&self) -> Box<IncrementalBucket<N, W, L>> {
        Box::new(IncrementalBucket::<N, W, L>::new(
            &self.haystacks,
            self.idxs,
            self.length,
        ))
    }

    pub fn add_haystack(
        &mut self,
        haystack: &'a str,
        idx: usize,
        buckets: &mut Vec<Box<dyn IncrementalBucketTrait>>,
    ) {
        assert!(haystack.len() <= W);
        if self.length == L {
            return;
        }

        self.haystacks[self.length] = haystack;
        self.idxs[self.length] = idx;
        self.length += 1;

        if self.length == L {
            buckets.push(self.build_bucket());
            self.idxs = [0; L];
            self.length = 0;
        }
    }

    pub fn finalize(&mut self, buckets: &mut Vec<Box<dyn IncrementalBucketTrait>>) {
        if self.length > 0 {
            buckets.push(self.build_bucket());
        }
    }
}
