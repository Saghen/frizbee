use super::bucket::FixedWidthBucket;
use super::Appendable;

use crate::one_shot::match_too_large;
use crate::prefilter::bitmask::string_to_bitmask;
use crate::smith_waterman::greedy::match_greedy;
use crate::{Match, Options};

/// Computes the Smith-Waterman score with affine gaps for the list of given targets.
///
/// You should call this function with as many targets as you have available as it will
/// automatically chunk the targets based on string length to avoid unnecessary computation
/// due to SIMD
pub fn match_list<S1: AsRef<str>, S2: AsRef<str>>(
    needle: S1,
    haystacks: &[S2],
    opts: Options,
) -> Vec<Match> {
    let mut matches = if opts.max_typos.is_none() {
        Vec::with_capacity(haystacks.len())
    } else {
        vec![]
    };

    match_list_impl(needle, haystacks, 0, opts, &mut matches);

    if opts.sort {
        #[cfg(feature = "parallel_sort")]
        {
            use rayon::prelude::*;
            matches.par_sort();
        }
        #[cfg(not(feature = "parallel_sort"))]
        matches.sort_unstable();
    }

    matches
}

pub(crate) fn match_list_impl<S1: AsRef<str>, S2: AsRef<str>, M: Appendable<Match>>(
    needle: S1,
    haystacks: &[S2],
    index_offset: u32,
    opts: Options,
    matches: &mut M,
) {
    assert!(
        (index_offset as usize) + haystacks.len() < (u32::MAX as usize),
        "haystack index overflow"
    );

    let needle = needle.as_ref();
    if needle.is_empty() {
        for (i, _) in haystacks.iter().enumerate() {
            matches.append(Match {
                index_in_haystack: (i as u32) + index_offset,
                score: 0,
                exact: false,
            });
        }
        return;
    }

    let needle_bitmask = string_to_bitmask(needle.as_bytes());

    let mut bucket_size_4 = FixedWidthBucket::<4>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_8 = FixedWidthBucket::<8>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_12 = FixedWidthBucket::<12>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_16 = FixedWidthBucket::<16>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_20 = FixedWidthBucket::<20>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_24 = FixedWidthBucket::<24>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_32 = FixedWidthBucket::<32>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_48 = FixedWidthBucket::<48>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_64 = FixedWidthBucket::<64>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_96 = FixedWidthBucket::<96>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_128 = FixedWidthBucket::<128>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_160 = FixedWidthBucket::<160>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_192 = FixedWidthBucket::<192>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_224 = FixedWidthBucket::<224>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_256 = FixedWidthBucket::<256>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_384 = FixedWidthBucket::<384>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_512 = FixedWidthBucket::<512>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_768 = FixedWidthBucket::<768>::new(needle, needle_bitmask, &opts);
    let mut bucket_size_1024 = FixedWidthBucket::<1024>::new(needle, needle_bitmask, &opts);

    // If max_typos is set, we can ignore any haystacks that are shorter than the needle
    // minus the max typos, since it's impossible for them to match
    let min_haystack_len = opts
        .max_typos
        .map(|max| needle.len() - (max as usize))
        .unwrap_or(0);

    for (i, haystack) in haystacks.iter().enumerate() {
        let i = i as u32 + index_offset;
        let haystack = haystack.as_ref();
        if haystack.len() < min_haystack_len {
            continue;
        }
        // fallback to greedy matching
        if match_too_large(needle, haystack) {
            let (score, _) = match_greedy(needle, haystack);
            matches.append(Match {
                index_in_haystack: i,
                score,
                exact: false,
            });
            continue;
        }

        // Pick the bucket to insert into based on the length of the haystack
        match haystack.len() {
            0..=4 => bucket_size_4.add_haystack(matches, haystack, i),
            5..=8 => bucket_size_8.add_haystack(matches, haystack, i),
            9..=12 => bucket_size_12.add_haystack(matches, haystack, i),
            13..=16 => bucket_size_16.add_haystack(matches, haystack, i),
            17..=20 => bucket_size_20.add_haystack(matches, haystack, i),
            21..=24 => bucket_size_24.add_haystack(matches, haystack, i),
            25..=32 => bucket_size_32.add_haystack(matches, haystack, i),
            33..=48 => bucket_size_48.add_haystack(matches, haystack, i),
            49..=64 => bucket_size_64.add_haystack(matches, haystack, i),
            65..=96 => bucket_size_96.add_haystack(matches, haystack, i),
            97..=128 => bucket_size_128.add_haystack(matches, haystack, i),
            129..=160 => bucket_size_160.add_haystack(matches, haystack, i),
            161..=192 => bucket_size_192.add_haystack(matches, haystack, i),
            193..=224 => bucket_size_224.add_haystack(matches, haystack, i),
            225..=256 => bucket_size_256.add_haystack(matches, haystack, i),
            257..=384 => bucket_size_384.add_haystack(matches, haystack, i),
            385..=512 => bucket_size_512.add_haystack(matches, haystack, i),
            513..=768 => bucket_size_768.add_haystack(matches, haystack, i),
            769..=1024 => bucket_size_1024.add_haystack(matches, haystack, i),

            // fallback to greedy matching
            _ => {
                let (score, _) = match_greedy(needle, haystack);
                matches.append(Match {
                    index_in_haystack: i,
                    score,
                    exact: false,
                });
                continue;
            }
        };
    }

    // Run processing on remaining haystacks in the buckets
    bucket_size_4.finalize(matches);
    bucket_size_8.finalize(matches);
    bucket_size_12.finalize(matches);
    bucket_size_16.finalize(matches);
    bucket_size_20.finalize(matches);
    bucket_size_24.finalize(matches);
    bucket_size_32.finalize(matches);
    bucket_size_48.finalize(matches);
    bucket_size_64.finalize(matches);
    bucket_size_96.finalize(matches);
    bucket_size_128.finalize(matches);
    bucket_size_160.finalize(matches);
    bucket_size_192.finalize(matches);
    bucket_size_224.finalize(matches);
    bucket_size_256.finalize(matches);
    bucket_size_384.finalize(matches);
    bucket_size_512.finalize(matches);
    bucket_size_768.finalize(matches);
    bucket_size_1024.finalize(matches);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let needle = "deadbe";
        let haystack = vec!["deadbeef", "deadbf", "deadbeefg", "deadbe"];

        let matches = match_list(
            needle,
            &haystack,
            Options {
                max_typos: None,
                ..Options::default()
            },
        );
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
            assert_eq!(haystack[m.index_in_haystack as usize], needle)
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
            assert_eq!(haystack[m.index_in_haystack as usize], needle)
        }
    }
}
