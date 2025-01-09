use crate::simd::smith_waterman;
use crate::Match;

pub(crate) trait Bucket<'a> {
    fn add_haystack(&mut self, haystack: &'a str, idx: usize);
    fn is_full(&self) -> bool;
    fn process(&mut self, matches: &mut [Option<Match>], needle: &str, _with_indices: bool);
    fn reset(&mut self);
}

pub(crate) struct FixedWidthBucket<'a, const W: usize> {
    width: usize,
    length: usize,
    haystacks: [&'a str; 16],
    idxs: [usize; 16],
}

impl<const W: usize> FixedWidthBucket<'_, W> {
    pub fn new() -> Self {
        FixedWidthBucket {
            width: if W <= 24 { 16 } else { 8 },
            length: 0,
            haystacks: [""; 16],
            idxs: [0; 16],
        }
    }
}

impl<'a, const W: usize> Bucket<'a> for FixedWidthBucket<'a, W>
where
    [(); W + 1]:,
{
    fn add_haystack(&mut self, haystack: &'a str, idx: usize) {
        assert!(haystack.len() <= W);
        if self.length == self.width {
            return;
        }
        self.haystacks[self.length] = haystack;
        self.idxs[self.length] = idx;
        self.length += 1;
    }

    fn is_full(&self) -> bool {
        self.length == self.width
    }

    fn process(&mut self, matches: &mut [Option<Match>], needle: &str, _with_indices: bool) {
        if self.length == 0 {
            return;
        }

        if W <= 24 {
            let scores = smith_waterman::<u8, W>(needle, &self.haystacks);
            for idx in 0..self.length {
                let score_idx = self.idxs[idx];
                matches[score_idx] = Some(Match {
                    index_in_haystack: score_idx,
                    index: score_idx,
                    score: scores[idx] as u16,
                    indices: None, //indices: bucket_indices.get(idx).cloned(),
                });
            }
        } else {
            let scores = smith_waterman::<u16, W>(needle, &self.haystacks);
            for idx in 0..self.length {
                let score_idx = self.idxs[idx];
                matches[score_idx] = Some(Match {
                    index_in_haystack: score_idx,
                    index: score_idx,
                    score: scores[idx],
                    indices: None, //indices: bucket_indices.get(idx).cloned(),
                });
            }
        }

        self.reset();
    }

    fn reset(&mut self) {
        self.length = 0;
    }
}
