use std::{
    cmp::Reverse,
    simd::{cmp::SimdOrd, Simd},
};

use crate::{
    simd::{smith_waterman_inner, HaystackChar, NeedleChar, SimdNum},
    Match, Options,
};

struct IncrementalBucket<N: SimdNum<L>, const L: usize>
where
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
{
    length: usize,
    width: usize,
    idxs: [usize; L],
    haystacks: Box<[HaystackChar<N, L>]>,
    score_matrix: Vec<Box<[Simd<N, L>]>>,
}

impl<N: SimdNum<L>, const L: usize> IncrementalBucket<N, L>
where
    N: SimdNum<L>,
    std::simd::LaneCount<L>: std::simd::SupportedLaneCount,
    std::simd::Simd<N, L>: crate::simd::SimdVec<N, L>,
    std::simd::Mask<N::Mask, L>: crate::simd::SimdMask<N, L>,
{
    pub fn new(haystacks: &[&str; L], idxs: [usize; L], length: usize) -> Self {
        let width = haystacks[0..length].iter().map(|&x| x.len()).max().unwrap();
        let haystack = (0..width)
            .into_iter()
            .map(|i| HaystackChar::from_haystacks(haystacks, i))
            .collect();
        Self {
            length,
            width,
            idxs,
            haystacks: haystack,
            score_matrix: vec![],
        }
    }

    #[inline]
    fn process(
        &mut self,
        prefix_to_keep: usize,
        new_needle_chars: &[NeedleChar<N, L>],
        matches: &mut Vec<Match>,
        min_score: u16,
        max_typos: Option<u16>,
    ) {
        self.score_matrix.truncate(prefix_to_keep);

        self.score_matrix.extend(std::iter::repeat_n(
            vec![N::ZERO_VEC; self.width].into_boxed_slice(),
            new_needle_chars.len(),
        ));

        for (i, &needle_char) in new_needle_chars.iter().enumerate() {
            let needle_idx = i + prefix_to_keep;

            let (prev_score_col, curr_score_col) = if needle_idx == 0 {
                (None, self.score_matrix[needle_idx].as_mut())
            } else {
                let (a, b) = self.score_matrix.split_at_mut(needle_idx);
                (Some(a[needle_idx - 1].as_ref()), b[0].as_mut())
            };

            smith_waterman_inner(
                self.width,
                needle_char,
                &self.haystacks,
                prev_score_col,
                curr_score_col,
            );
        }

        let mut all_time_max_score = N::ZERO_VEC;
        for score_col in self.score_matrix.iter() {
            for score in score_col {
                all_time_max_score = score.simd_max(all_time_max_score);
            }
        }

        // TODO: DRY w/ smith_waterman
        let scores: [u16; L] = std::array::from_fn(|i| {
            let score = all_time_max_score[i].into();
            // TODO: exact match bonus - this is going to be tricky becayse raw haystacks aren't
            // currently stored. perhaps simd the comparison?
            // if haystacks[i] == needle_str {
            //     score += EXACT_MATCH_BONUS;
            // }
            score
        });

        // TODO: typos

        #[allow(clippy::needless_range_loop)]
        for idx in 0..self.length {
            let score = scores[idx];
            if score < min_score {
                continue;
            }
            let score_idx = self.idxs[idx];
            matches.push(Match {
                index_in_haystack: score_idx,
                score: scores[idx],
                indices: None,
                exact: false,
            });
        }
    }
}

pub struct IncrementalMatcher {
    needle: Option<String>,
    num_haystacks: usize,
    small_buckets: Box<[IncrementalBucket<u8, 16>]>,
    large_buckets: Box<[IncrementalBucket<u16, 8>]>,
}

impl IncrementalMatcher {
    pub fn new(haystacks: &Vec<&str>) -> Self {
        // group haystacks into buckets by length

        // TODO: prefiltering? If yes, then haystacks can't be put into buckets yet
        let mut haystacks_data = haystacks
            .iter()
            .enumerate()
            .map(|(idx, &data)| (idx, data.len()))
            .collect::<Box<[_]>>();
        haystacks_data.sort_by_key(|&(_, len)| len);

        let split_idx = haystacks_data
            .iter()
            .position(|&(_, len)| len > 24)
            .unwrap_or(haystacks.len());
        let (small_haystacks_data, large_haystacks_data) = haystacks_data.split_at(split_idx);
        let mut small_buckets = vec![];
        for chunk in small_haystacks_data.chunks(16) {
            let mut haystacks_chunk = [""; 16];
            let mut idx_chunk = [0; 16];
            for (i, &(idx, _)) in chunk.iter().enumerate() {
                haystacks_chunk[i] = haystacks[idx];
                idx_chunk[i] = idx;
            }
            small_buckets.push(IncrementalBucket::new(
                &haystacks_chunk,
                idx_chunk,
                chunk.len(),
            ));
        }
        let mut large_buckets = vec![];
        for chunk in large_haystacks_data.chunks(8) {
            let mut haystacks_chunk = [""; 8];
            let mut idx_chunk = [0; 8];
            for (i, &(idx, _)) in chunk.iter().enumerate() {
                haystacks_chunk[i] = haystacks[idx];
                idx_chunk[i] = idx;
            }
            large_buckets.push(IncrementalBucket::new(
                &haystacks_chunk,
                idx_chunk,
                chunk.len(),
            ));
        }

        Self {
            needle: None,
            num_haystacks: haystacks.len(),
            small_buckets: small_buckets.into_boxed_slice(),
            large_buckets: large_buckets.into_boxed_slice(),
        }
    }

    pub fn match_needle(&mut self, needle: &str, opts: Options) -> Vec<Match> {
        if needle.is_empty() {
            todo!();
        }

        let mut matches = Vec::with_capacity(self.num_haystacks);

        let common_prefix_len = self
            .needle
            .as_ref()
            .map(|prev_needle| {
                needle
                    .as_bytes()
                    .iter()
                    .zip(prev_needle.as_bytes())
                    .take_while(|&(&a, &b)| a == b)
                    .count()
            })
            .unwrap_or(0);

        self.process(common_prefix_len, needle, &mut matches, opts);
        self.needle = Some(needle.to_owned());

        if opts.stable_sort {
            matches.sort_by_key(|mtch| Reverse(mtch.score));
        } else if opts.unstable_sort {
            matches.sort_unstable_by_key(|mtch| Reverse(mtch.score));
        }

        matches
    }

    fn process(
        &mut self,
        prefix_to_keep: usize,
        needle: &str,
        matches: &mut Vec<Match>,
        opts: Options,
    ) {
        let needle = &needle.as_bytes()[prefix_to_keep..];
        let needle_chars = needle
            .into_iter()
            .map(|&x| NeedleChar::new(x))
            .collect::<Box<[_]>>();

        for bucket in self.small_buckets.iter_mut() {
            bucket.process(
                prefix_to_keep,
                &needle_chars,
                matches,
                opts.min_score,
                opts.max_typos,
            );
        }

        let needle_chars = needle
            .into_iter()
            .map(|&x| NeedleChar::new(x as u16))
            .collect::<Box<[_]>>();
        for bucket in self.large_buckets.iter_mut() {
            bucket.process(
                prefix_to_keep,
                &needle_chars,
                matches,
                opts.min_score,
                opts.max_typos,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#const::*;

    const CHAR_SCORE: u16 = MATCH_SCORE + MATCHING_CASE_BONUS;

    fn get_score(needle: &str, haystack: &str) -> u16 {
        let mut matcher = IncrementalMatcher::new(&vec![haystack]);
        matcher.match_needle(needle, Options::default())[0].score
    }

    #[test]
    fn test_score_basic() {
        assert_eq!(get_score("b", "abc"), CHAR_SCORE);
        assert_eq!(get_score("c", "abc"), CHAR_SCORE);
    }

    #[test]
    fn test_score_prefix() {
        assert_eq!(get_score("a", "abc"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("a", "aabc"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("a", "babc"), CHAR_SCORE);
    }

    #[test]
    fn test_score_exact_match() {
        assert_eq!(
            get_score("a", "a"),
            CHAR_SCORE + EXACT_MATCH_BONUS + PREFIX_BONUS
        );
        assert_eq!(
            get_score("abc", "abc"),
            3 * CHAR_SCORE + EXACT_MATCH_BONUS + PREFIX_BONUS
        );
        assert_eq!(get_score("ab", "abc"), 2 * CHAR_SCORE + PREFIX_BONUS);
        // assert_eq!(run_single("abc", "ab"), 2 * CHAR_SCORE + PREFIX_BONUS);
    }

    #[test]
    fn test_score_delimiter() {
        assert_eq!(get_score("b", "a-b"), CHAR_SCORE + DELIMITER_BONUS);
        assert_eq!(get_score("a", "a-b-c"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("b", "a--b"), CHAR_SCORE + DELIMITER_BONUS);
        assert_eq!(get_score("c", "a--bc"), CHAR_SCORE);
        assert_eq!(get_score("a", "-a--bc"), CHAR_SCORE);
        assert_eq!(get_score("-", "a-bc"), CHAR_SCORE);
        assert_eq!(get_score("-", "a--bc"), CHAR_SCORE + DELIMITER_BONUS);
    }

    #[test]
    fn test_score_affine_gap() {
        assert_eq!(
            get_score("test", "Uterst"),
            CHAR_SCORE * 4 - GAP_OPEN_PENALTY
        );
        assert_eq!(
            get_score("test", "Uterrst"),
            CHAR_SCORE * 4 - GAP_OPEN_PENALTY - GAP_EXTEND_PENALTY
        );
    }

    #[test]
    fn test_score_capital_bonus() {
        assert_eq!(get_score("a", "A"), MATCH_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("A", "Aa"), CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("D", "forDist"), CHAR_SCORE + CAPITALIZATION_BONUS);
        assert_eq!(get_score("D", "foRDist"), CHAR_SCORE);
    }

    #[test]
    fn test_score_prefix_beats_delimiter() {
        assert!(get_score("swap", "swap(test)") > get_score("swap", "iter_swap(test)"),);
    }
}
