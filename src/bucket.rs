use crate::r#const::SIMD_WIDTH;
use crate::simd::*;
use crate::Match;

pub(crate) struct Bucket<'a> {
    length: usize,
    haystacks: [&'a str; SIMD_WIDTH],
    idxs: [usize; SIMD_WIDTH],
    smith_waterman_func: fn(&str, &[&str]) -> [u16; SIMD_WIDTH],
}

impl<'a> Bucket<'a> {
    pub fn new(string_length: usize) -> Self {
        Bucket {
            length: 0,
            haystacks: [""; SIMD_WIDTH],
            idxs: [0; SIMD_WIDTH],
            smith_waterman_func: match string_length {
                4 => smith_waterman_inter_simd_4,
                8 => smith_waterman_inter_simd_8,
                12 => smith_waterman_inter_simd_12,
                16 => smith_waterman_inter_simd_16,
                20 => smith_waterman_inter_simd_20,
                24 => smith_waterman_inter_simd_24,
                32 => smith_waterman_inter_simd_32,
                48 => smith_waterman_inter_simd_48,
                64 => smith_waterman_inter_simd_64,
                96 => smith_waterman_inter_simd_96,
                128 => smith_waterman_inter_simd_128,
                160 => smith_waterman_inter_simd_160,
                192 => smith_waterman_inter_simd_192,
                224 => smith_waterman_inter_simd_224,
                256 => smith_waterman_inter_simd_256,
                384 => smith_waterman_inter_simd_384,
                512 => smith_waterman_inter_simd_512,
                _ => panic!("Invalid string length"),
            },
        }
    }

    pub fn add_haystack(&mut self, haystack: &'a str, idx: usize) {
        if self.length == SIMD_WIDTH {
            return;
        }
        self.haystacks[self.length] = haystack;
        self.idxs[self.length] = idx;
        self.length += 1;
    }

    pub fn is_full(&self) -> bool {
        self.length == SIMD_WIDTH
    }

    pub fn process(&mut self, matches: &mut [Option<Match>], needle: &str, _with_indices: bool) {
        if self.length == 0 {
            return;
        }

        let bucket_scores = (self.smith_waterman_func)(needle, &self.haystacks);
        //let bucket_indices = if with_indices {
        //    vec![]
        //    //char_indices_from_scores(
        //    //    &bucket_score_matrix
        //    //        .iter()
        //    //        .flatten()
        //    //        .copied()
        //    //        .collect::<Vec<_>>(),
        //    //    &bucket_scores,
        //    //    self.haystack_len(),
        //    //)
        //} else {
        //    vec![]
        //};

        for idx in 0..self.length {
            let score_idx = self.idxs[idx];
            matches[score_idx] = Some(Match {
                index_in_haystack: score_idx,
                index: score_idx,
                score: bucket_scores[idx],
                indices: None, //indices: bucket_indices.get(idx).cloned(),
            });
        }
        self.reset();
    }

    pub fn reset(&mut self) {
        self.length = 0;
    }
}
