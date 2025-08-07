use std::{
    arch::x86_64::{_mm_set1_epi8, _mm256_loadu2_m128i},
    cell::RefCell,
    simd::{Simd, cmp::SimdOrd},
};

use super::interleave::interleave;
use super::prefilter::match_haystack_unordered_insensitive;
use crate::{
    Match, Scoring,
    incremental::interleave::{deinterleave_u16, interleave_u16},
    smith_waterman::simd::{
        HaystackChar, NeedleChar, smith_waterman_inner, typos_from_score_matrix,
    },
};

struct IncrementalHaystack<const W: usize> {
    pub data: [u8; W],
    pub score_matrix: RefCell<Vec<[u16; W]>>,
    pub idx: u32,
    /// Index of the needle character that caused the item to be filtered out
    pub filtered_at: u32,
}

impl<const W: usize> IncrementalHaystack<W> {
    pub fn new(idx: u32, haystack: &str) -> Self {
        let mut data = [0u8; W];
        data[0..haystack.len()].copy_from_slice(haystack.as_bytes());

        Self {
            data,
            score_matrix: RefCell::new(vec![]),
            idx,
            filtered_at: u32::MAX,
        }
    }

    pub fn truncate_to(&self, idx: u32) {
        self.score_matrix.borrow_mut().truncate(idx as usize);
    }

    pub fn push_scores(&self, scores: [u16; W]) {
        self.score_matrix.borrow_mut().push(scores);
    }
}

pub(crate) struct IncrementalBucket<const W: usize> {
    haystacks: Vec<IncrementalHaystack<W>>,
}

impl<const W: usize> IncrementalBucket<W> {
    pub fn new() -> Self {
        Self { haystacks: vec![] }
    }

    pub fn add_haystack(&mut self, haystack_str: &str, idx: u32) {
        self.haystacks
            .push(IncrementalHaystack::new(idx, haystack_str));
    }

    pub fn process(
        &mut self,
        prefix_to_keep: usize,
        needle_chars: &[u8],
        matches: &mut Vec<Match>,
        scoring: &Scoring,
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

        let haystack_iter = self
            .haystacks
            .iter()
            .filter(|h| h.filtered_at > prefix_to_keep as u32)
            .filter(|h| unsafe {
                match_haystack_unordered_insensitive(&needle_chars_avx2, &h.data)
            });

        // TODO: handle remainder
        for haystacks in haystack_iter.array_chunks::<16>() {
            let haystack_chars = interleave::<W>(haystacks.map(|h| h.data)).map(HaystackChar::new);

            let mut prev_score_col = if prefix_to_keep > 0 {
                Some(interleave_u16(
                    haystacks.map(|h| h.score_matrix.borrow()[prefix_to_keep - 1]),
                ))
            } else {
                None
            };
            let mut curr_score_col = [Simd::splat(0); W];

            for haystack in haystacks.iter() {
                haystack.truncate_to(prefix_to_keep as u32);
            }

            for &needle_char in needle_chars_simd.iter() {
                smith_waterman_inner(
                    0,
                    W,
                    needle_char,
                    &haystack_chars,
                    prev_score_col.as_ref().map(|s| s.as_slice()),
                    &mut curr_score_col,
                    scoring,
                );

                let scores = deinterleave_u16::<W>(curr_score_col);
                for (haystack, scores) in haystacks.iter().zip(scores.into_iter()) {
                    haystack.push_scores(scores);
                }

                prev_score_col = Some(curr_score_col);
                curr_score_col = [Simd::splat(0); W];
            }
        }
    }
}
