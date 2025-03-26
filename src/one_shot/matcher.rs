use super::bucket::{Bucket, FixedWidthBucket};
use crate::prefilter::bitmask::string_to_bitmask;
use crate::{Match, Options};
use std::cmp::Reverse;

/// Computes the Smith-Waterman score with affine gaps for the list of given targets.
///
/// You should call this function with as many targets as you have available as it will
/// automatically chunk the targets based on string length to avoid unnecessary computation
/// due to SIMD
pub fn match_list(needle: &str, haystacks: &[&str], opts: Options) -> Vec<Match> {
    if needle.is_empty() {
        return haystacks
            .iter()
            .enumerate()
            .map(|(i, _)| Match {
                index_in_haystack: i,
                score: 0,
                exact: false,
                indices: None,
            })
            .collect();
    }

    let needle_bitmask = string_to_bitmask(needle.as_bytes());

    // Lazy bucket initialization - buckets are created only when needed
    let mut bucket_size_4 = FixedWidthBucket::<u8, 4, 16>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_8 = FixedWidthBucket::<u8, 8, 16>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_12 = FixedWidthBucket::<u8, 12, 16>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_16 = FixedWidthBucket::<u8, 16, 16>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_20 = FixedWidthBucket::<u8, 20, 16>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_24 = FixedWidthBucket::<u8, 24, 16>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_32 = FixedWidthBucket::<u16, 32, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_48 = FixedWidthBucket::<u16, 48, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_64 = FixedWidthBucket::<u16, 64, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_96 = FixedWidthBucket::<u16, 96, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_128 = FixedWidthBucket::<u16, 128, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_160 = FixedWidthBucket::<u16, 160, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_192 = FixedWidthBucket::<u16, 192, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_224 = FixedWidthBucket::<u16, 224, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_256 = FixedWidthBucket::<u16, 256, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_384 = FixedWidthBucket::<u16, 384, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_512 = FixedWidthBucket::<u16, 512, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_768 = FixedWidthBucket::<u16, 768, 8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_1024 = FixedWidthBucket::<u16, 1024, 8>::new(needle, needle_bitmask, &opts);

    let mut matches = if opts.max_typos == None {
        Vec::with_capacity(haystacks.len())
    } else {
        vec![]
    };

    for (i, haystack) in haystacks.iter().enumerate() {
        // Pick the bucket to insert into based on the length of the haystack
        match haystack.len() {
            0..=4 => bucket_size_4.add_haystack(&mut matches, haystack, i),
            5..=8 => bucket_size_8.add_haystack(&mut matches, haystack, i),
            9..=12 => bucket_size_12.add_haystack(&mut matches, haystack, i),
            13..=16 => bucket_size_16.add_haystack(&mut matches, haystack, i),
            17..=20 => bucket_size_20.add_haystack(&mut matches, haystack, i),
            21..=24 => bucket_size_24.add_haystack(&mut matches, haystack, i),
            25..=32 => bucket_size_32.add_haystack(&mut matches, haystack, i),
            33..=48 => bucket_size_48.add_haystack(&mut matches, haystack, i),
            49..=64 => bucket_size_64.add_haystack(&mut matches, haystack, i),
            65..=96 => bucket_size_96.add_haystack(&mut matches, haystack, i),
            97..=128 => bucket_size_128.add_haystack(&mut matches, haystack, i),
            129..=160 => bucket_size_160.add_haystack(&mut matches, haystack, i),
            161..=192 => bucket_size_192.add_haystack(&mut matches, haystack, i),
            193..=224 => bucket_size_224.add_haystack(&mut matches, haystack, i),
            225..=256 => bucket_size_256.add_haystack(&mut matches, haystack, i),
            257..=384 => bucket_size_384.add_haystack(&mut matches, haystack, i),
            385..=512 => bucket_size_512.add_haystack(&mut matches, haystack, i),
            513..=768 => bucket_size_768.add_haystack(&mut matches, haystack, i),
            769..=1024 => bucket_size_1024.add_haystack(&mut matches, haystack, i),
            // TODO: implement greedy fallback strategy
            _ => continue,
        };
    }

    // Run processing on remaining haystacks in the buckets
    bucket_size_4.finalize(&mut matches);
    bucket_size_8.finalize(&mut matches);
    bucket_size_12.finalize(&mut matches);
    bucket_size_16.finalize(&mut matches);
    bucket_size_20.finalize(&mut matches);
    bucket_size_24.finalize(&mut matches);
    bucket_size_32.finalize(&mut matches);
    bucket_size_48.finalize(&mut matches);
    bucket_size_64.finalize(&mut matches);
    bucket_size_96.finalize(&mut matches);
    bucket_size_128.finalize(&mut matches);
    bucket_size_160.finalize(&mut matches);
    bucket_size_192.finalize(&mut matches);
    bucket_size_224.finalize(&mut matches);
    bucket_size_256.finalize(&mut matches);
    bucket_size_384.finalize(&mut matches);
    bucket_size_512.finalize(&mut matches);
    bucket_size_768.finalize(&mut matches);
    bucket_size_1024.finalize(&mut matches);

    // Sorting
    if opts.stable_sort {
        matches.sort_by_key(|mtch| Reverse(mtch.score));
    } else if opts.unstable_sort {
        matches.sort_unstable_by_key(|mtch| Reverse(mtch.score));
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let needle = "deadbe";
        let haystack = vec!["deadbeef", "deadbf", "deadbeefg", "deadbe"];

        let matches = match_list(needle, &haystack, Options::default());
        assert_eq!(matches.len(), 4);
        assert_eq!(matches[0].index_in_haystack, 3);
        assert_eq!(matches[1].index_in_haystack, 0);
        assert_eq!(matches[2].index_in_haystack, 2);
        assert_eq!(matches[3].index_in_haystack, 1);
    }

    #[test]
    fn test_no_typos() {
        let needle = "deadbe";
        let haystack = vec!["deadbeef", "deadbf", "deadbeefg", "deadbe"];

        let matches = match_list(
            needle,
            &haystack,
            Options {
                max_typos: Some(0),
                ..Options::default()
            },
        );
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_exact_match() {
        let needle = "deadbe";
        let haystack = vec!["deadbeef", "deadbf", "deadbeefg", "deadbe"];

        let matches = match_list(needle, &haystack, Options::default());

        let exact_matches = matches.iter().filter(|m| m.exact).collect::<Vec<&Match>>();
        assert_eq!(exact_matches.len(), 1);
        assert_eq!(exact_matches[0].index_in_haystack, 3);
        for m in &exact_matches {
            assert_eq!(haystack[m.index_in_haystack], needle)
        }
    }

    #[test]
    fn test_exact_matches() {
        let needle = "deadbe";
        let haystack = vec![
            "deadbe",
            "deadbeef",
            "deadbe",
            "deadbf",
            "deadbe",
            "deadbeefg",
            "deadbe",
        ];

        let matches = match_list(needle, &haystack, Options::default());

        let exact_matches = matches.iter().filter(|m| m.exact).collect::<Vec<&Match>>();
        assert_eq!(exact_matches.len(), 4);
        for m in &exact_matches {
            assert_eq!(haystack[m.index_in_haystack], needle)
        }
    }
}
