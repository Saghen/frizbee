use std::{mem::MaybeUninit, sync::Arc};

use crate::{Match, Options};

use super::{matcher::match_list_impl, vec::BatchedLockFreeQueue};

pub trait Appendable<T> {
    fn append(&mut self, value: T);
}

impl<T> Appendable<T> for Vec<T> {
    fn append(&mut self, value: T) {
        self.push(value);
    }
}

struct VecSlice<'a, T> {
    slice: &'a mut [MaybeUninit<T>],
    idx: usize,
}

impl<'a, T> VecSlice<'a, T> {
    fn new(slice: &'a mut [MaybeUninit<T>]) -> Self {
        Self { slice, idx: 0 }
    }
}

impl<'a, T> Appendable<T> for VecSlice<'a, T> {
    fn append(&mut self, value: T) {
        self.slice[self.idx].write(value);
        self.idx += 1;
    }
}

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
    // TODO: 5000 was chosen arbitrarily, need to benchmark
    let thread_count = (haystacks.len() / 5000).min(max_threads);
    let items_per_thread = haystacks.len().div_ceil(thread_count);

    let batch_size = 256;
    let capacity = opts
        .max_typos
        // TODO: make the BatchedLockFreeQueue expandable
        .map(|_| haystacks.len() / 19)
        .unwrap_or(haystacks.len());
    let matches = Arc::new(BatchedLockFreeQueue::new(
        capacity,
        batch_size,
        thread_count,
    ));

    std::thread::scope(|s| {
        for haystacks in haystacks.chunks(items_per_thread) {
            let needle = needle.as_ref().to_owned();
            let mut matches = matches.clone();
            s.spawn(move || match_list_impl(needle, haystacks, opts, &mut matches));
        }
    });

    Arc::try_unwrap(matches).unwrap().into_vec()

    // let mut matches: Vec<MaybeUninit<Match>> = Vec::with_capacity(haystacks.len());
    // unsafe {
    //     matches.set_len(haystacks.len());
    // }
    //
    // std::thread::scope(|s| {
    //     let mut matches_slice = matches.as_mut_slice();
    //     for haystacks in haystacks.chunks_exact(items_per_thread) {
    //         let needle = needle.as_ref().to_owned();
    //
    //         let (mut chunk_slice, remaining_slice) = matches_slice.split_at_mut(haystacks.len());
    //         matches_slice = remaining_slice;
    //
    //         s.spawn(move || {
    //             let mut chunk_vec_slice = VecSlice::new(&mut chunk_slice);
    //             match_list_impl(needle, haystacks, opts, &mut chunk_vec_slice);
    //         });
    //     }
    // });
    //
    // vec![]

    // use rayon::prelude::*;
    // let needle = needle.as_ref();
    //
    // let mut matches = haystacks
    //     .par_chunks_exact(5000)
    //     .flat_map(|haystacks| {
    //         let needle = needle.to_owned();
    //         match_list(needle, haystacks, opts)
    //     })
    //     .collect::<Vec<_>>();

    // if opts.sort {
    //     matches.sort_unstable_by_key(|mtch| Reverse(mtch.score));
    // }
}
