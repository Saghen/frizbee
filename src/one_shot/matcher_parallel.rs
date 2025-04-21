use std::cmp::Reverse;

use crate::{Match, Options};

use super::matcher::match_list_impl;

pub fn flatten_optional_vec<T>(mut v: Vec<Option<T>>) -> Vec<T> {
    v.sort_unstable_by_key(|mtch| (mtch.is_some()));

    let first_none = v.iter().position(|m| m.is_none());
    if let Some(first_none) = first_none {
        v.truncate(first_none);
    }

    // Now convert Vec<Option<T>> to Vec<T> in-place
    let ptr = v.as_mut_ptr() as *mut T;
    let len = v.len();
    let cap = v.capacity();

    // Prevent the old vector from being dropped
    std::mem::forget(v);

    // Create a Vec<T> from the raw parts
    unsafe { Vec::from_raw_parts(ptr, len, cap) }
}

/// Computes the Smith-Waterman score with affine gaps for the list of given targets with
/// multithreading.
///
/// You should call this function with as many targets as you have available as it will
/// automatically chunk the targets based on string length to avoid unnecessary computation
/// due to SIMD
pub fn match_list_parallel<S1: AsRef<str>, S2: AsRef<str> + Sync + Send>(
    needle: S1,
    haystacks: &[S2],
    opts: Options,
    max_threads: usize,
) -> Vec<Match> {
    // TODO: 20000 was chosen arbitrarily, need to benchmark
    let thread_count = (haystacks.len() / 20000).min(max_threads);
    let items_per_thread = haystacks.len() / thread_count;

    let mut matches = vec![None; haystacks.len()];
    std::thread::scope(|s| {
        let mut matches_slice = matches.as_mut_slice();

        for haystacks in haystacks.chunks_exact(items_per_thread) {
            let (chunk_slice, remaining_slice) = matches_slice.split_at_mut(haystacks.len());
            matches_slice = remaining_slice;
            let needle = needle.as_ref().to_owned();
            s.spawn(move || {
                match_list_impl(needle, haystacks, opts, chunk_slice);
            });
        }
    });

    let mut matches = flatten_optional_vec(matches);
    if opts.sort {
        matches.sort_unstable_by_key(|mtch| Reverse(mtch.score));
    }
    matches
}
