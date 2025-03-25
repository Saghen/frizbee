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
    let mut bucket_size_4: Option<FixedWidthBucket<u8, 4, 16>> = None;
    let mut bucket_size_8: Option<FixedWidthBucket<u8, 8, 16>> = None;
    let mut bucket_size_12: Option<FixedWidthBucket<u8, 12, 16>> = None;
    let mut bucket_size_16: Option<FixedWidthBucket<u8, 16, 16>> = None;
    let mut bucket_size_20: Option<FixedWidthBucket<u8, 20, 16>> = None;
    let mut bucket_size_24: Option<FixedWidthBucket<u8, 24, 16>> = None;
    let mut bucket_size_32: Option<FixedWidthBucket<u16, 32, 8>> = None;
    let mut bucket_size_48: Option<FixedWidthBucket<u16, 48, 8>> = None;
    let mut bucket_size_64: Option<FixedWidthBucket<u16, 64, 8>> = None;
    let mut bucket_size_96: Option<FixedWidthBucket<u16, 96, 8>> = None;
    let mut bucket_size_128: Option<FixedWidthBucket<u16, 128, 8>> = None;
    let mut bucket_size_160: Option<FixedWidthBucket<u16, 160, 8>> = None;
    let mut bucket_size_192: Option<FixedWidthBucket<u16, 192, 8>> = None;
    let mut bucket_size_224: Option<FixedWidthBucket<u16, 224, 8>> = None;
    let mut bucket_size_256: Option<FixedWidthBucket<u16, 256, 8>> = None;
    let mut bucket_size_384: Option<FixedWidthBucket<u16, 384, 8>> = None;
    let mut bucket_size_512: Option<FixedWidthBucket<u16, 512, 8>> = None;
    let mut bucket_size_768: Option<FixedWidthBucket<u16, 768, 8>> = None;
    let mut bucket_size_1024: Option<FixedWidthBucket<u16, 1024, 8>> = None;

    let mut matches = if opts.max_typos == None {
        Vec::with_capacity(haystacks.len())
    } else {
        vec![]
    };

    for (i, haystack) in haystacks.iter().enumerate() {
        // Pick the bucket to insert into based on the length of the haystack
        match haystack.len() {
            0..=4 => {
                if bucket_size_4.is_none() {
                    bucket_size_4 = Some(FixedWidthBucket::<u8, 4, 16>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_4.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            5..=8 => {
                if bucket_size_8.is_none() {
                    bucket_size_8 = Some(FixedWidthBucket::<u8, 8, 16>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_8.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            9..=12 => {
                if bucket_size_12.is_none() {
                    bucket_size_12 = Some(FixedWidthBucket::<u8, 12, 16>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_12.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            13..=16 => {
                if bucket_size_16.is_none() {
                    bucket_size_16 = Some(FixedWidthBucket::<u8, 16, 16>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_16.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            17..=20 => {
                if bucket_size_20.is_none() {
                    bucket_size_20 = Some(FixedWidthBucket::<u8, 20, 16>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_20.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            21..=24 => {
                if bucket_size_24.is_none() {
                    bucket_size_24 = Some(FixedWidthBucket::<u8, 24, 16>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_24.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            25..=32 => {
                if bucket_size_32.is_none() {
                    bucket_size_32 = Some(FixedWidthBucket::<u16, 32, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_32.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            33..=48 => {
                if bucket_size_48.is_none() {
                    bucket_size_48 = Some(FixedWidthBucket::<u16, 48, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_48.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            49..=64 => {
                if bucket_size_64.is_none() {
                    bucket_size_64 = Some(FixedWidthBucket::<u16, 64, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_64.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            65..=96 => {
                if bucket_size_96.is_none() {
                    bucket_size_96 = Some(FixedWidthBucket::<u16, 96, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_96.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            97..=128 => {
                if bucket_size_128.is_none() {
                    bucket_size_128 = Some(FixedWidthBucket::<u16, 128, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_128.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            129..=160 => {
                if bucket_size_160.is_none() {
                    bucket_size_160 = Some(FixedWidthBucket::<u16, 160, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_160.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            161..=192 => {
                if bucket_size_192.is_none() {
                    bucket_size_192 = Some(FixedWidthBucket::<u16, 192, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_192.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            193..=224 => {
                if bucket_size_224.is_none() {
                    bucket_size_224 = Some(FixedWidthBucket::<u16, 224, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_224.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            225..=256 => {
                if bucket_size_256.is_none() {
                    bucket_size_256 = Some(FixedWidthBucket::<u16, 256, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_256.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            257..=384 => {
                if bucket_size_384.is_none() {
                    bucket_size_384 = Some(FixedWidthBucket::<u16, 384, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_384.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            385..=512 => {
                if bucket_size_512.is_none() {
                    bucket_size_512 = Some(FixedWidthBucket::<u16, 512, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_512.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            513..=768 => {
                if bucket_size_768.is_none() {
                    bucket_size_768 = Some(FixedWidthBucket::<u16, 768, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_768.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            769..=1024 => {
                if bucket_size_1024.is_none() {
                    bucket_size_1024 = Some(FixedWidthBucket::<u16, 1024, 8>::new(needle, needle_bitmask, &opts));
                }
                bucket_size_1024.as_mut().unwrap().add_haystack(&mut matches, haystack, i);
            },
            // TODO: implement greedy fallback strategy
            _ => continue,
        };
    }

    // Run processing on remaining haystacks in the buckets
    if let Some(ref mut bucket) = bucket_size_4 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_8 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_12 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_16 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_20 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_24 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_32 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_48 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_64 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_96 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_128 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_160 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_192 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_224 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_256 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_384 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_512 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_768 { bucket.finalize(&mut matches); }
    if let Some(ref mut bucket) = bucket_size_1024 { bucket.finalize(&mut matches); }

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
