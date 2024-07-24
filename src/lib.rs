#![feature(portable_simd)]

extern crate memchr;

mod bucket;
pub mod r#const;
mod prefilter;
pub mod simd;

use bucket::Bucket;
use prefilter::prefilter_ascii;
use std::cmp::Reverse;

#[derive(Debug, Clone, Default)]
pub struct Match {
    pub index: usize,
    pub score: u16,
    pub indices: Option<Vec<usize>>,
}

/// Computes the Smith-Waterman score with affine gaps for the list of given targets.
/// You should call this function with as many targets as you have available as it will
/// automatically chunk the targets based on string length to avoid unnecessary computation
/// due to SIMD
pub fn match_list(needle: &str, haystacks: &[&str], opts: Options) -> Vec<Match> {
    if needle.is_empty() {
        return haystacks
            .iter()
            .enumerate()
            .map(|(i, _)| Match {
                index: i,
                score: 0,
                indices: None,
            })
            .collect();
    }

    let needle = needle.to_ascii_lowercase();

    let mut haystacks = haystacks.to_vec();

    // Filters
    if opts.prefilter {
        haystacks = haystacks
            .iter()
            .filter(|target| {
                if target.len() <= needle.len() {
                    return false;
                }

                let result = prefilter_ascii(needle.as_bytes(), target.as_bytes());
                if let Some((start, _greedy_end, end)) = result {
                    needle.len() != (end - start)
                } else {
                    false
                }
            })
            .copied()
            .collect();
    }

    let mut buckets = [
        Bucket::new(4),
        Bucket::new(8),
        Bucket::new(12),
        Bucket::new(16),
        Bucket::new(24),
        Bucket::new(32),
        Bucket::new(48),
        Bucket::new(64),
        Bucket::new(96),
        Bucket::new(128),
        Bucket::new(160),
        Bucket::new(192),
        Bucket::new(224),
        Bucket::new(256),
    ];
    let mut matches = vec![Match::default(); haystacks.len()];

    for (i, haystack) in haystacks.iter().enumerate() {
        // Pick the bucket to insert into based on the length of the haystack
        let bucket_idx = match haystack.len() {
            0..=4 => 0,
            5..=8 => 1,
            9..=12 => 2,
            13..=16 => 3,
            17..=24 => 4,
            25..=32 => 5,
            33..=48 => 6,
            49..=64 => 7,
            65..=96 => 8,
            97..=128 => 9,
            129..=160 => 10,
            161..=192 => 11,
            193..=224 => 12,
            225..=256 => 13,
            257..=384 => 14,
            385..=512 => 15,
            _ => 15, // TODO: should just return score = 0
        };

        let bucket = &mut buckets[bucket_idx];
        bucket.add_haystack(haystack, i);

        if bucket.is_full() {
            bucket.process(&mut matches, &needle, opts.indices);
        }
    }

    // Iterate over the bucket with remaining elements
    for bucket in buckets.iter_mut() {
        bucket.process(&mut matches, &needle, opts.indices);
    }

    if opts.min_score > 0 {
        matches.retain(|mtch| mtch.score >= opts.min_score);
    }
    if opts.stable_sort {
        matches.sort_by_key(|mtch| Reverse(mtch.score));
    } else if opts.unstable_sort {
        matches.sort_unstable_by_key(|mtch| Reverse(mtch.score));
    }

    matches
}

pub struct Options {
    /// Populate score matrix and perform traceback to get the indices of the matching characters
    pub indices: bool,
    /// Uses fzf's prefilter algorithm to remove any haystacks that do not include all of the
    /// characters in the needle. This may remove many haystacks that contain good matches
    pub prefilter: bool,
    /// Sort the results while maintaining the original order of the haystacks
    pub stable_sort: bool,
    /// Sort the results without maintaining the original order of the haystacks (much faster on
    /// long lists)
    pub unstable_sort: bool,
    /// Minimum score to return a result
    pub min_score: u16,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            indices: false,
            prefilter: false,
            stable_sort: true,
            unstable_sort: false,
            min_score: 2,
        }
    }
}
