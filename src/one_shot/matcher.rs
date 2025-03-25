use super::bucket::{Bucket, FixedWidthBucket};
use crate::prefilter::bitmask::{string_to_bitmask, string_to_bitmask_simd};
use crate::prefilter::memchr;
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

    let mut matches = Vec::with_capacity(haystacks.len());

    let mut buckets: [Box<dyn Bucket>; 19] = [
        Box::new(FixedWidthBucket::<u8, 4, 16>::new()),
        Box::new(FixedWidthBucket::<u8, 8, 16>::new()),
        Box::new(FixedWidthBucket::<u8, 12, 16>::new()),
        Box::new(FixedWidthBucket::<u8, 16, 16>::new()),
        Box::new(FixedWidthBucket::<u8, 20, 16>::new()),
        Box::new(FixedWidthBucket::<u8, 24, 16>::new()),
        Box::new(FixedWidthBucket::<u16, 32, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 48, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 64, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 96, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 128, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 160, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 192, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 224, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 256, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 384, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 512, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 768, 8>::new()),
        Box::new(FixedWidthBucket::<u16, 1024, 8>::new()),
    ];

    let needle_bitmask = string_to_bitmask(needle.as_bytes());

    for (i, haystack) in haystacks.iter().enumerate() {
        // Pick the bucket to insert into based on the length of the haystack
        let bucket_idx = match haystack.len() {
            0..=4 => 0,
            5..=8 => 1,
            9..=12 => 2,
            13..=16 => 3,
            17..=20 => 4,
            21..=24 => 5,
            25..=32 => 6,
            33..=48 => 7,
            49..=64 => 8,
            65..=96 => 9,
            97..=128 => 10,
            129..=160 => 11,
            161..=192 => 12,
            193..=224 => 13,
            225..=256 => 14,
            257..=384 => 15,
            385..=512 => 16,
            513..=768 => 17,
            769..=1024 => 18,
            // TODO: should return score = 0 or fallback to prefilter
            _ => continue,
        };

        // Perform a fast path with bitmasking if there are no typos or 1 typo
        // This makes the algorithm 6x faster in the case of no matches
        // in the haystack
        let prefilter = !opts.prefilter
            || match opts.max_typos {
                // Use memchr for prefiltering when the haystack is too long
                Some(0) if bucket_idx >= 5 => memchr::prefilter(needle, *haystack),
                Some(1) if bucket_idx >= 4 => memchr::prefilter_with_typo(needle, *haystack),

                // Othewrise, use bitmasking
                Some(0) => {
                    needle_bitmask & string_to_bitmask_simd(haystack.as_bytes()) == needle_bitmask
                }
                // TODO: skip this when typos > 2?
                Some(max) => {
                    (needle_bitmask & string_to_bitmask_simd(haystack.as_bytes()) ^ needle_bitmask)
                        .count_ones()
                        <= max as u32
                }
                _ => true,
            };
        if !prefilter {
            continue;
        }

        let bucket = &mut buckets[bucket_idx];
        bucket.add_haystack(haystack, i);

        if bucket.is_full() {
            bucket.process(&mut matches, needle, opts.min_score, opts.max_typos);
        }
    }

    // Iterate over the bucket with remaining elements
    for (bucket_idx, bucket) in buckets.iter_mut().enumerate() {
        let max_typos = match opts.max_typos {
            // if we used memchr, we can be certain there's no typos
            Some(0) if bucket_idx >= 5 => None,
            Some(1) if bucket_idx >= 4 => None,

            _ => opts.max_typos,
        };

        bucket.process(&mut matches, needle, opts.min_score, max_typos);
    }

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
