use std::cmp::Reverse;

use super::bucket::IncrementalBucket;
use crate::{Config, Match};

pub struct Matcher {
    needle: Option<String>,

    bucket_size_8: IncrementalBucket<8>,
    bucket_size_12: IncrementalBucket<12>,
    bucket_size_16: IncrementalBucket<16>,
    bucket_size_20: IncrementalBucket<20>,
    bucket_size_24: IncrementalBucket<24>,
    bucket_size_32: IncrementalBucket<32>,
    bucket_size_48: IncrementalBucket<48>,
    bucket_size_64: IncrementalBucket<64>,
    bucket_size_96: IncrementalBucket<96>,
    bucket_size_128: IncrementalBucket<128>,
    bucket_size_160: IncrementalBucket<160>,
    bucket_size_192: IncrementalBucket<192>,
    bucket_size_224: IncrementalBucket<224>,
    bucket_size_256: IncrementalBucket<256>,
    bucket_size_384: IncrementalBucket<384>,
    bucket_size_512: IncrementalBucket<512>,
}

impl Matcher {
    pub fn new<S: AsRef<str> + Ord + Clone>(haystacks: &[S]) -> Self {
        let mut haystacks = haystacks.iter().collect::<Vec<_>>();
        haystacks.sort();

        // group haystacks into buckets by length
        let mut bucket_size_8 = IncrementalBucket::<8>::new();
        let mut bucket_size_12 = IncrementalBucket::<12>::new();
        let mut bucket_size_16 = IncrementalBucket::<16>::new();
        let mut bucket_size_20 = IncrementalBucket::<20>::new();
        let mut bucket_size_24 = IncrementalBucket::<24>::new();
        let mut bucket_size_32 = IncrementalBucket::<32>::new();
        let mut bucket_size_48 = IncrementalBucket::<48>::new();
        let mut bucket_size_64 = IncrementalBucket::<64>::new();
        let mut bucket_size_96 = IncrementalBucket::<96>::new();
        let mut bucket_size_128 = IncrementalBucket::<128>::new();
        let mut bucket_size_160 = IncrementalBucket::<160>::new();
        let mut bucket_size_192 = IncrementalBucket::<192>::new();
        let mut bucket_size_224 = IncrementalBucket::<224>::new();
        let mut bucket_size_256 = IncrementalBucket::<256>::new();
        let mut bucket_size_384 = IncrementalBucket::<384>::new();
        let mut bucket_size_512 = IncrementalBucket::<512>::new();

        for (i, haystack) in haystacks.iter().enumerate() {
            let i = i as u32;
            let haystack = haystack.as_ref();
            match haystack.len() {
                0..=8 => bucket_size_8.add_haystack(haystack, i),
                9..=12 => bucket_size_12.add_haystack(haystack, i),
                13..=16 => bucket_size_16.add_haystack(haystack, i),
                17..=20 => bucket_size_20.add_haystack(haystack, i),
                21..=24 => bucket_size_24.add_haystack(haystack, i),
                25..=32 => bucket_size_32.add_haystack(haystack, i),
                33..=48 => bucket_size_48.add_haystack(haystack, i),
                49..=64 => bucket_size_64.add_haystack(haystack, i),
                65..=96 => bucket_size_96.add_haystack(haystack, i),
                97..=128 => bucket_size_128.add_haystack(haystack, i),
                129..=160 => bucket_size_160.add_haystack(haystack, i),
                161..=192 => bucket_size_192.add_haystack(haystack, i),
                193..=224 => bucket_size_224.add_haystack(haystack, i),
                225..=256 => bucket_size_256.add_haystack(haystack, i),
                257..=384 => bucket_size_384.add_haystack(haystack, i),
                385..=512 => bucket_size_512.add_haystack(haystack, i),
                // TODO: should return score = 0 or fallback to prefilter
                _ => continue,
            };
        }

        Self {
            needle: None,

            bucket_size_8,
            bucket_size_12,
            bucket_size_16,
            bucket_size_20,
            bucket_size_24,
            bucket_size_32,
            bucket_size_48,
            bucket_size_64,
            bucket_size_96,
            bucket_size_128,
            bucket_size_160,
            bucket_size_192,
            bucket_size_224,
            bucket_size_256,
            bucket_size_384,
            bucket_size_512,
        }
    }

    pub fn reset(&mut self) {
        self.needle = None;

        self.bucket_size_8.reset();
        self.bucket_size_12.reset();
        self.bucket_size_16.reset();
        self.bucket_size_20.reset();
        self.bucket_size_24.reset();
        self.bucket_size_32.reset();
        self.bucket_size_48.reset();
        self.bucket_size_64.reset();
        self.bucket_size_96.reset();
        self.bucket_size_128.reset();
        self.bucket_size_160.reset();
        self.bucket_size_192.reset();
        self.bucket_size_224.reset();
        self.bucket_size_256.reset();
        self.bucket_size_384.reset();
        self.bucket_size_512.reset();
    }

    pub fn match_needle<S: AsRef<str>>(&mut self, needle: S, config: Config) -> Vec<Match> {
        let needle = needle.as_ref();
        if needle.is_empty() {
            todo!()
        }

        let mut matches = vec![];

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

        self.process(
            common_prefix_len,
            &needle.as_bytes()[common_prefix_len..],
            &mut matches,
            config.clone(),
        );
        self.needle = Some(needle.to_owned());

        if config.sort {
            matches.sort_unstable_by_key(|mtch| Reverse(mtch.score));
        }

        matches
    }

    fn process(
        &mut self,
        common_prefix_len: usize,
        needle: &[u8],
        matches: &mut Vec<Match>,
        config: Config,
    ) {
        self.bucket_size_8.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_12.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_16.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_20.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_24.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_32.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_48.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_64.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_96.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_128.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_160.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_192.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_224.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_256.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_384.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
        self.bucket_size_512.process(
            common_prefix_len,
            needle,
            matches,
            &config.scoring,
            config.max_typos,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#const::*;

    const CHAR_SCORE: u16 = MATCH_SCORE + MATCHING_CASE_BONUS;

    fn get_score(needle: &str, haystack: &str) -> u16 {
        let mut matcher = Matcher::new(&[haystack]);
        matcher.match_needle(needle, Config::default())[0].score
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
    fn test_score_offset_prefix() {
        // Give prefix bonus on second char if the first char isn't a letter
        assert_eq!(get_score("a", "-a"), CHAR_SCORE + OFFSET_PREFIX_BONUS);
        assert_eq!(get_score("-a", "-ab"), 2 * CHAR_SCORE + PREFIX_BONUS);
        assert_eq!(get_score("a", "'a"), CHAR_SCORE + OFFSET_PREFIX_BONUS);
        assert_eq!(get_score("a", "Ba"), CHAR_SCORE);
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
        assert_eq!(get_score("a", "-a--bc"), CHAR_SCORE + OFFSET_PREFIX_BONUS);
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
