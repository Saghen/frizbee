#![feature(portable_simd)]

mod bitmask;
mod bucket;
pub mod r#const;
mod match_;
mod prefilter;
mod reference;
pub mod score_matrix;
pub mod simd;

pub use crate::match_::Match;

use crate::score_matrix::*;
use bitmask::{string_to_bitmask, string_to_bitmask_simd};
use bucket::{Bucket, FixedWidthBucket};
use r#const::SIMD_WIDTH;
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

    let mut buckets: [Box<dyn Bucket>; 17] = [
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
            // TODO: should return score = 0 or fallback to prefilter
            _ => continue,
        };

        // Perform a fast path with memchr if there are no typos or 1 typo
        // This makes the algorithm 6x faster in the case of no matches
        // in the haystack
        let prefilter = !opts.prefilter
            || match opts.max_typos {
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
    for bucket in buckets.iter_mut() {
        bucket.process(&mut matches, needle, opts.min_score, opts.max_typos);
    }

    // Sorting
    if opts.stable_sort {
        matches.sort_by_key(|mtch| Reverse(mtch.score));
    } else if opts.unstable_sort {
        matches.sort_unstable_by_key(|mtch| Reverse(mtch.score));
    }

    matches
}

pub fn match_list_for_matched_indices(needle: &str, haystacks: &[&str]) -> Vec<Vec<usize>> {
    // TODO: sort by length
    let haystacks = haystacks.to_vec();

    let mut matched_indices_arr = Vec::with_capacity(haystacks.len());

    for haystack_idx in (0..haystacks.len()).step_by(SIMD_WIDTH) {
        let length = (haystacks.len().saturating_sub(haystack_idx)).min(SIMD_WIDTH);
        let mut sliced_haystacks = [""; SIMD_WIDTH];
        sliced_haystacks[0..length]
            .copy_from_slice(&haystacks[haystack_idx..haystack_idx + length]);

        let (score_matrices, max_score_locations) =
            smith_waterman_with_scoring_matrix(needle, &sliced_haystacks);
        let haystack_slice_matched_indices =
            char_indices_from_scores(score_matrices, max_score_locations);

        for matched_indices in haystack_slice_matched_indices.into_iter().take(length) {
            matched_indices_arr.push(matched_indices);
        }
    }

    matched_indices_arr
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Performs prefiltering when the max number of typos is <= 1, which drastically improves
    /// performance when most of the haystack does not match (<10% matching)
    pub prefilter: bool,
    /// Minimum score of an item to return a result. Generally, needle.len() * 6 will  be a good
    /// default
    pub min_score: u16,
    /// The maximum number of characters missing from the needle, before an item in the
    /// haystack is filtered out
    pub max_typos: Option<u16>,
    /// Sort the results while maintaining the original order of the haystacks
    pub stable_sort: bool,
    /// Sort the results without maintaining the original order of the haystacks (much faster on
    /// long lists)
    pub unstable_sort: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            prefilter: true,
            min_score: 0,
            max_typos: None,
            stable_sort: true,
            unstable_sort: false,
        }
    }
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

        assert_eq!(matches.iter().filter(|m| m.exact).count(), 1);
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

        assert_eq!(matches.iter().filter(|m| m.exact).count(), 4);
    }
}
