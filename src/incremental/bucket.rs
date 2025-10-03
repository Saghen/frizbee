use std::{
    arch::x86_64::{__m256i, _mm_set1_epi8, _mm256_loadu2_m128i},
    simd::{LaneCount, Simd, SupportedLaneCount},
};

use super::interleave::interleave;
use super::prefilter::match_haystack_unordered_insensitive;
use crate::{
    Match, Scoring,
    smith_waterman::simd::{HaystackChar, NeedleChar, smith_waterman_inner},
};

const L: usize = 16;

#[derive(Debug, Clone)]
pub(crate) struct Haystack<const W: usize> {
    pub data: [u8; W],
    pub filtered_at: u32,
    pub index: u32,
}

impl<const W: usize> Haystack<W> {
    pub fn new(haystack: &str, index: u32) -> Self {
        let mut data = [0u8; W];
        data[0..haystack.len()].copy_from_slice(haystack.as_bytes());

        Self {
            data,
            filtered_at: u32::MAX,
            index,
        }
    }

    pub fn reset(&mut self) {
        self.filtered_at = u32::MAX;
    }
}

impl<const W: usize> Default for Haystack<W> {
    fn default() -> Self {
        Self {
            data: [0u8; W],
            filtered_at: u32::MAX,
            index: u32::MAX,
        }
    }
}

pub(crate) struct HaystackSimd<const W: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub haystacks: Vec<Haystack<W>>,
    pub simd: [HaystackChar<L>; W],
    pub score_matrix: Vec<[Simd<u16, L>; W]>,
}

impl<const W: usize> HaystackSimd<W> {
    pub fn new(haystacks: Vec<Haystack<W>>) -> Self {
        let data = haystacks
            .iter()
            .map(|h| h.data)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            haystacks,
            simd: interleave::<W>(data).map(HaystackChar::new),
            score_matrix: Vec::with_capacity(16),
        }
    }

    pub fn truncate_to(&mut self, idx: u32) {
        self.score_matrix.truncate(idx as usize);
    }

    pub fn push_scores(&mut self, scores: [Simd<u16, L>; W]) {
        self.score_matrix.push(scores);
    }

    pub fn prefilter(&mut self, prefix_to_keep: u32, needle_chars: &[__m256i]) -> Vec<u8> {
        let mut matched_indices = Vec::with_capacity(L);

        for (i, h) in self.haystacks.iter_mut().enumerate() {
            if h.filtered_at <= prefix_to_keep {
                continue;
            }

            if unsafe { match_haystack_unordered_insensitive(needle_chars, &h.data) } {
                h.filtered_at = u32::MAX;
                matched_indices.push(i as u8);
            } else {
                h.filtered_at = prefix_to_keep + needle_chars.len() as u32;
            }
        }

        matched_indices

        // // TODO: update filtered_at
        // self.haystacks
        //     .iter_mut()
        //     .enumerate()
        //     .filter(|(_, h)| h.filtered_at > prefix_to_keep)
        //     .filter(|(_, h)| unsafe { match_haystack_unordered_insensitive(needle_chars, &h.data) })
        //     .map(|(i, _)| i as u8)
        //     .collect()
    }

    pub fn reset(&mut self) {
        for h in self.haystacks.iter_mut() {
            h.reset();
        }
    }
}

pub(crate) struct IncrementalBucket<const W: usize> {
    ungrouped_haystacks: Vec<Haystack<W>>,
    haystacks: Vec<HaystackSimd<W>>,
}

impl<const W: usize> IncrementalBucket<W> {
    pub fn new() -> Self {
        Self {
            ungrouped_haystacks: vec![],
            haystacks: vec![],
        }
    }

    pub fn add_haystack(&mut self, haystack_str: &str, idx: u32) {
        self.ungrouped_haystacks
            .push(Haystack::new(haystack_str, idx));
        if self.ungrouped_haystacks.len() >= L {
            self.haystacks.push(HaystackSimd::new(
                self.ungrouped_haystacks.drain(..).collect(),
            ));
        }
    }

    pub fn reset(&mut self) {
        for h in self.haystacks.iter_mut() {
            h.reset();
        }
    }

    pub fn process(
        &mut self,
        prefix_to_keep: usize,
        needle_chars: &[u8],
        matches: &mut Vec<Match>,
        scoring: &Scoring,
        max_typos: Option<u16>,
    ) {
        if self.haystacks.is_empty() {
            return;
        }

        let needle_chars_avx2 = needle_chars
            .iter()
            .map(|&c| unsafe {
                _mm256_loadu2_m128i(
                    &_mm_set1_epi8(c.to_ascii_uppercase() as i8),
                    &_mm_set1_epi8(c.to_ascii_lowercase() as i8),
                )
            })
            .collect::<Vec<_>>();

        let needle_chars_simd = needle_chars
            .iter()
            .map(|&c| NeedleChar::new(c as u16))
            .collect::<Vec<_>>();

        // TODO: handle remainder
        let mut skipped_haystacks = vec![];
        for (i, haystacks) in self.haystacks.iter_mut().enumerate() {
            let matched_indices = haystacks.prefilter(prefix_to_keep as u32, &needle_chars_avx2);
            if matched_indices.len() <= L / 2 {
                skipped_haystacks.push((i, matched_indices));
                continue;
            }

            let mut prev_score_col = if prefix_to_keep > 0 {
                Some(haystacks.score_matrix[prefix_to_keep - 1])
            } else {
                None
            };
            let mut curr_score_col = [Simd::splat(0); W];

            haystacks.truncate_to(prefix_to_keep as u32);

            for &needle_char in needle_chars_simd.iter() {
                smith_waterman_inner(
                    0,
                    W,
                    needle_char,
                    &haystacks.simd,
                    prev_score_col.as_ref().map(|s| s.as_slice()),
                    &mut curr_score_col,
                    scoring,
                );

                haystacks.push_scores(curr_score_col);

                prev_score_col = Some(curr_score_col);
                curr_score_col = [Simd::splat(0); W];
            }

            for idx in matched_indices {
                matches.push(Match {
                    exact: false,
                    index: haystacks.haystacks[idx as usize].index,
                    score: 0,
                });
            }
        }

        let mut new_haystacks = vec![];
        let mut to_combine: Vec<Haystack<W>> = vec![];

        for (i, matched_indices) in skipped_haystacks {
            for idx in matched_indices.iter() {
                to_combine.push(self.haystacks[i].haystacks[*idx as usize].clone());

                if to_combine.len() == L {
                    new_haystacks.push(HaystackSimd::new(to_combine));
                    to_combine = vec![];
                }
            }
        }

        for haystacks in new_haystacks.iter_mut() {
            let matched_indices = haystacks.prefilter(prefix_to_keep as u32, &needle_chars_avx2);

            let mut prev_score_col = if prefix_to_keep > 0 {
                Some(haystacks.score_matrix[prefix_to_keep - 1])
            } else {
                None
            };
            let mut curr_score_col = [Simd::splat(0); W];

            haystacks.truncate_to(prefix_to_keep as u32);

            for &needle_char in needle_chars_simd.iter() {
                smith_waterman_inner(
                    0,
                    W,
                    needle_char,
                    &haystacks.simd,
                    prev_score_col.as_ref().map(|s| s.as_slice()),
                    &mut curr_score_col,
                    scoring,
                );

                haystacks.push_scores(curr_score_col);

                prev_score_col = Some(curr_score_col);
                curr_score_col = [Simd::splat(0); W];
            }

            for idx in matched_indices {
                matches.push(Match {
                    exact: false,
                    index: haystacks.haystacks[idx as usize].index,
                    score: 0,
                });
            }
        }
    }
}
