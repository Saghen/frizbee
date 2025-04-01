use std::marker::PhantomData;

use crate::prefilter::bitmask::string_to_bitmask_simd;
use crate::prefilter::memchr;
use crate::smith_waterman::simd::{
    char_indices_from_scores, smith_waterman, typos_from_score_matrix, SimdMask, SimdNum, SimdVec,
};
use crate::{Match, Options};

#[derive(Debug, Clone, Copy)]
enum PrefilterMethod {
    None,
    Memchr,
    Bitmask,
}

pub(crate) trait Bucket<'a> {
    fn add_haystack(&mut self, matches: &mut Vec<Match>, haystack: &'a str, idx: usize);
    fn finalize(&mut self, matches: &mut Vec<Match>);
}

pub(crate) struct FixedWidthBucket<'a, N: SimdNum<L>, const W: usize, const L: usize>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    length: usize,
    needle: &'a str,
    needle_bitmask: u64,
    haystacks: [&'a str; L],
    idxs: [usize; L],
    min_score: u16,
    max_typos: Option<u16>,
    prefilter: PrefilterMethod,
    matched_indices: bool,
    _phantom: PhantomData<N>,
}

impl<'a, N: SimdNum<L>, const W: usize, const L: usize> FixedWidthBucket<'a, N, W, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    std::simd::Simd<N, L>: SimdVec<N, L>,
    std::simd::Mask<N::Mask, L>: SimdMask<N, L>,
{
    pub fn new(needle: &'a str, needle_bitmask: u64, opts: &Options) -> Self {
        FixedWidthBucket {
            length: 0,
            needle,
            needle_bitmask,
            haystacks: [""; L],
            idxs: [0; L],
            min_score: opts.min_score,
            max_typos: opts.max_typos,
            prefilter: match (opts.prefilter, opts.max_typos) {
                (false, _) => PrefilterMethod::None,
                (_, None) => PrefilterMethod::None,
                (true, Some(0)) if L >= 24 => PrefilterMethod::Memchr,
                (true, Some(1)) if L >= 20 => PrefilterMethod::Memchr,
                (true, _) => PrefilterMethod::Bitmask,
            },
            matched_indices: opts.matched_indices,
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
    fn add_haystack(&mut self, matches: &mut Vec<Match>, haystack: &'a str, idx: usize) {
        if !matches!(self.prefilter, PrefilterMethod::None) {
            let matched = match (self.prefilter, self.max_typos) {
                (PrefilterMethod::Memchr, Some(0)) => memchr::prefilter(self.needle, haystack),
                (PrefilterMethod::Memchr, Some(1)) => {
                    memchr::prefilter_with_typo(self.needle, haystack)
                }

                (PrefilterMethod::Bitmask, Some(0)) => {
                    self.needle_bitmask & string_to_bitmask_simd(haystack.as_bytes())
                        == self.needle_bitmask
                }
                // TODO: skip this when typos > 2?
                (PrefilterMethod::Bitmask, Some(max)) => {
                    (self.needle_bitmask & string_to_bitmask_simd(haystack.as_bytes())
                        ^ self.needle_bitmask)
                        .count_ones()
                        <= max as u32
                }
                _ => true,
            };
            if !matched {
                return;
            }
        }

        self.haystacks[self.length] = haystack;
        self.idxs[self.length] = idx;
        self.length += 1;

        if self.length == L {
            self.finalize(matches);
        }
    }

    fn finalize(&mut self, matches: &mut Vec<Match>) {
        if self.length == 0 {
            return;
        }

        let (scores, score_matrix, exact_matches) =
            smith_waterman::<N, W, L>(self.needle, &self.haystacks);

        let typos = self
            .max_typos
            .map(|_| typos_from_score_matrix::<N, W, L>(&score_matrix));

        let mut matched_indices = self
            .matched_indices
            .then(|| char_indices_from_scores(&score_matrix).into_iter());

        #[allow(clippy::needless_range_loop)]
        for idx in 0..self.length {
            let score = scores[idx];
            if score < self.min_score {
                continue;
            }

            // Memchr guarantees the number of typos is <= max_typos so no need to check
            if !matches!(self.prefilter, PrefilterMethod::Memchr) {
                if let Some(max_typos) = self.max_typos {
                    if typos.is_some_and(|typos| typos[idx] > max_typos) {
                        continue;
                    }
                }
            }

            let indices = matched_indices.as_mut().and_then(|iter| iter.next());

            let score_idx = self.idxs[idx];
            matches.push(Match {
                index_in_haystack: score_idx,
                score: scores[idx],
                exact: exact_matches[idx],
                indices,
            });
        }

        self.length = 0;
    }
}
