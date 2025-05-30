use crate::prefilter::bitmask::string_to_bitmask_simd;
use crate::prefilter::memchr;
use crate::smith_waterman::simd::{
    smith_waterman, typos_from_score_matrix, SimdMask, SimdNum, SimdVec,
};
use crate::{Match, Options};

use super::Appendable;

#[derive(Debug, Clone, Copy)]
enum PrefilterMethod {
    None,
    Memchr,
    Bitmask,
}

#[derive(Debug)]
pub(crate) struct FixedWidthBucket<'a, const W: usize> {
    has_avx512: bool,
    has_avx2: bool,
    length: usize,
    needle: &'a str,
    needle_bitmask: u64,
    haystacks: [&'a str; 32],
    idxs: [u32; 32],
    min_score: u16,
    max_typos: Option<u16>,
    prefilter: PrefilterMethod,
}

impl<'a, const W: usize> FixedWidthBucket<'a, W> {
    pub fn new(needle: &'a str, needle_bitmask: u64, opts: &Options) -> Self {
        FixedWidthBucket {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            has_avx512: is_x86_feature_detected!("avx512f")
                && is_x86_feature_detected!("avx512bitalg"),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            has_avx2: is_x86_feature_detected!("avx2"),

            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            has_avx512: false,
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            has_avx2: false,

            length: 0,
            needle,
            needle_bitmask,
            haystacks: [""; 32],
            idxs: [0; 32],
            min_score: opts.min_score,
            max_typos: opts.max_typos,
            prefilter: match (opts.prefilter, opts.max_typos) {
                (true, Some(0)) if W >= 24 => PrefilterMethod::Memchr,
                (true, Some(1)) if W >= 20 => PrefilterMethod::Memchr,
                // TODO: disable on long haystacks? arbitrarily picked 48 for now
                (true, _) if W < 48 => PrefilterMethod::Bitmask,
                _ => PrefilterMethod::None,
            },
        }
    }

    pub fn add_haystack(
        &mut self,
        matches: &mut dyn Appendable<Match>,
        haystack: &'a str,
        idx: u32,
    ) {
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

        match self.length {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            32 if self.has_avx512 => unsafe { self.finalize_512(matches) },
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            16 if self.has_avx2 && !self.has_avx512 => unsafe { self.finalize_256(matches) },
            8 if !self.has_avx2 && !self.has_avx512 => self.finalize_128(matches),
            _ => {}
        }
    }

    pub fn finalize(&mut self, matches: &mut dyn Appendable<Match>) {
        match self.length {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            17.. if self.has_avx512 => unsafe { self.finalize_512(matches) },
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            9.. if self.has_avx2 => unsafe { self.finalize_256(matches) },
            0.. => self.finalize_128(matches),
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx512f", enable = "avx512bitalg")]
    unsafe fn finalize_512(&mut self, matches: &mut dyn Appendable<Match>) {
        self._finalize::<u16, 32>(matches);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    unsafe fn finalize_256(&mut self, matches: &mut dyn Appendable<Match>) {
        self._finalize::<u16, 16>(matches);
    }

    fn finalize_128(&mut self, matches: &mut dyn Appendable<Match>) {
        self._finalize::<u16, 8>(matches);
    }

    #[inline(always)]
    fn _finalize<N: SimdNum<L>, const L: usize>(&mut self, matches: &mut dyn Appendable<Match>)
    where
        std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
        std::simd::Simd<N, L>: SimdVec<N, L>,
        std::simd::Mask<N::Mask, L>: SimdMask<N, L>,
    {
        if self.length == 0 {
            return;
        }

        let (scores, score_matrix, exact_matches) = smith_waterman::<N, W, L>(
            self.needle,
            &self.haystacks.get(0..L).unwrap().try_into().unwrap(),
            self.max_typos,
        );

        let typos = self
            .max_typos
            .map(|max_typos| typos_from_score_matrix::<N, W, L>(&score_matrix, max_typos));

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

            let score_idx = self.idxs[idx];
            matches.append(Match {
                index_in_haystack: score_idx,
                score: scores[idx],
                exact: exact_matches[idx],
            });
        }

        self.length = 0;
    }
}
