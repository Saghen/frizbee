use std::cmp::Reverse;

use super::{bucket::IncrementalBucketTrait, bucket_collection::IncrementalBucketCollection};
use crate::{Match, Options};

pub struct IncrementalMatcher {
    needle: Option<String>,
    num_haystacks: usize,
    buckets: Vec<Box<dyn IncrementalBucketTrait>>,
}

impl IncrementalMatcher {
    pub fn new<S: AsRef<str>>(haystacks: &[S]) -> Self {
        // group haystacks into buckets by length

        // TODO: prefiltering? If yes, then haystacks can't be put into buckets yet

        let mut buckets: Vec<Box<dyn IncrementalBucketTrait>> = vec![];

        let mut collection_size_4 = IncrementalBucketCollection::<'_, u16, 4, 8>::new();
        let mut collection_size_8 = IncrementalBucketCollection::<'_, u16, 8, 8>::new();
        let mut collection_size_12 = IncrementalBucketCollection::<'_, u16, 12, 8>::new();
        let mut collection_size_16 = IncrementalBucketCollection::<'_, u16, 16, 8>::new();
        let mut collection_size_20 = IncrementalBucketCollection::<'_, u16, 20, 8>::new();
        let mut collection_size_24 = IncrementalBucketCollection::<'_, u16, 24, 8>::new();
        let mut collection_size_32 = IncrementalBucketCollection::<'_, u16, 32, 8>::new();
        let mut collection_size_48 = IncrementalBucketCollection::<'_, u16, 48, 8>::new();
        let mut collection_size_64 = IncrementalBucketCollection::<'_, u16, 64, 8>::new();
        let mut collection_size_96 = IncrementalBucketCollection::<'_, u16, 96, 8>::new();
        let mut collection_size_128 = IncrementalBucketCollection::<'_, u16, 128, 8>::new();
        let mut collection_size_160 = IncrementalBucketCollection::<'_, u16, 160, 8>::new();
        let mut collection_size_192 = IncrementalBucketCollection::<'_, u16, 192, 8>::new();
        let mut collection_size_224 = IncrementalBucketCollection::<'_, u16, 224, 8>::new();
        let mut collection_size_256 = IncrementalBucketCollection::<'_, u16, 256, 8>::new();
        let mut collection_size_384 = IncrementalBucketCollection::<'_, u16, 384, 8>::new();
        let mut collection_size_512 = IncrementalBucketCollection::<'_, u16, 512, 8>::new();
        let mut collection_size_768 = IncrementalBucketCollection::<'_, u16, 768, 8>::new();
        let mut collection_size_1024 = IncrementalBucketCollection::<'_, u16, 1024, 8>::new();

        for (i, haystack) in haystacks.iter().enumerate() {
            let haystack = haystack.as_ref();
            match haystack.len() {
                0..=4 => collection_size_4.add_haystack(haystack, i, &mut buckets),
                5..=8 => collection_size_8.add_haystack(haystack, i, &mut buckets),
                9..=12 => collection_size_12.add_haystack(haystack, i, &mut buckets),
                13..=16 => collection_size_16.add_haystack(haystack, i, &mut buckets),
                17..=20 => collection_size_20.add_haystack(haystack, i, &mut buckets),
                21..=24 => collection_size_24.add_haystack(haystack, i, &mut buckets),
                25..=32 => collection_size_32.add_haystack(haystack, i, &mut buckets),
                33..=48 => collection_size_48.add_haystack(haystack, i, &mut buckets),
                49..=64 => collection_size_64.add_haystack(haystack, i, &mut buckets),
                65..=96 => collection_size_96.add_haystack(haystack, i, &mut buckets),
                97..=128 => collection_size_128.add_haystack(haystack, i, &mut buckets),
                129..=160 => collection_size_160.add_haystack(haystack, i, &mut buckets),
                161..=192 => collection_size_192.add_haystack(haystack, i, &mut buckets),
                193..=224 => collection_size_224.add_haystack(haystack, i, &mut buckets),
                225..=256 => collection_size_256.add_haystack(haystack, i, &mut buckets),
                257..=384 => collection_size_384.add_haystack(haystack, i, &mut buckets),
                385..=512 => collection_size_512.add_haystack(haystack, i, &mut buckets),
                513..=768 => collection_size_768.add_haystack(haystack, i, &mut buckets),
                769..=1024 => collection_size_1024.add_haystack(haystack, i, &mut buckets),
                // TODO: should return score = 0 or fallback to prefilter
                _ => continue,
            };
        }

        collection_size_4.finalize(&mut buckets);
        collection_size_8.finalize(&mut buckets);
        collection_size_12.finalize(&mut buckets);
        collection_size_16.finalize(&mut buckets);
        collection_size_20.finalize(&mut buckets);
        collection_size_24.finalize(&mut buckets);
        collection_size_32.finalize(&mut buckets);
        collection_size_48.finalize(&mut buckets);
        collection_size_64.finalize(&mut buckets);
        collection_size_96.finalize(&mut buckets);
        collection_size_128.finalize(&mut buckets);
        collection_size_160.finalize(&mut buckets);
        collection_size_192.finalize(&mut buckets);
        collection_size_224.finalize(&mut buckets);
        collection_size_256.finalize(&mut buckets);
        collection_size_384.finalize(&mut buckets);
        collection_size_512.finalize(&mut buckets);
        collection_size_768.finalize(&mut buckets);
        collection_size_1024.finalize(&mut buckets);

        Self {
            needle: None,
            num_haystacks: haystacks.len(),
            buckets,
        }
    }

    pub fn match_needle<S: AsRef<str>>(&mut self, needle: S, opts: Options) -> Vec<Match> {
        let needle = needle.as_ref();
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

        if opts.sort {
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

        for bucket in self.buckets.iter_mut() {
            bucket.process(
                prefix_to_keep,
                needle,
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
        let mut matcher = IncrementalMatcher::new(&[haystack]);
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
    #[ignore = "Incremental matcher doesn't support exact matches until we implement them in SIMD"]
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
    }

    #[test]
    fn test_score_no_delimiter_for_delimiter_chars() {
        assert_eq!(get_score("-", "a-bc"), CHAR_SCORE);
        assert_eq!(get_score("-", "a--bc"), CHAR_SCORE);
        assert!(get_score("a_b", "a_bb") > get_score("a_b", "a__b"));
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
