use std::sync::Arc;

use super::{match_list, Appendable};
use crate::one_shot::matcher::match_list_impl;
use crate::{Match, Options};

mod expandable_queue;
mod fixed_queue;

use expandable_queue::ExpandableBatchedQueue;
use fixed_queue::FixedBatchedQueue;

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
    // TODO: ideally, we'd change this based on the average length of items in the haystack and the
    // legnth of the needle. Perhaps random sampling would work well?
    let min_items_per_thread = match opts.max_typos {
        Some(0) => 5000,
        // Slower prefilter makes is ~2x slower than no typos
        Some(1) => 3000,
        // Slower than ignoring typos since we need to perform backtracking
        Some(_) => 2000,
        None => 2500,
    };

    let thread_count = (haystacks.len() / min_items_per_thread).min(max_threads);
    if thread_count == 1 {
        return match_list(needle, haystacks, opts);
    }

    // TODO: pick based on number of items and threads
    let batch_size = 512;

    let queue = if opts.max_typos.is_some() {
        BatchedQueue::Expandable(ExpandableBatchedQueue::new(batch_size, thread_count))
    } else {
        BatchedQueue::Fixed(FixedBatchedQueue::new(
            haystacks.len(),
            batch_size,
            thread_count,
        ))
    };
    let queue = Arc::new(queue);

    let items_per_thread = haystacks.len().div_ceil(thread_count);
    std::thread::scope(|s| {
        for haystacks in haystacks.chunks(items_per_thread) {
            let needle = needle.as_ref().to_owned();
            let mut matches = queue.clone();
            s.spawn(move || match_list_impl(needle, haystacks, opts, &mut matches));
        }
    });

    Arc::try_unwrap(queue).unwrap().into_vec()
}

#[derive(Debug)]
enum BatchedQueue<T> {
    Fixed(FixedBatchedQueue<T>),
    Expandable(ExpandableBatchedQueue<T>),
}

impl<T> BatchedQueue<T> {
    fn into_vec(self) -> Vec<T> {
        match self {
            BatchedQueue::Fixed(q) => q.into_vec(),
            BatchedQueue::Expandable(q) => q.into_vec(),
        }
    }
}

impl<T> Appendable<T> for Arc<BatchedQueue<T>> {
    fn append(&mut self, value: T) {
        match unsafe { Arc::get_mut_unchecked(self) } {
            BatchedQueue::Fixed(q) => q.push(value),
            BatchedQueue::Expandable(q) => q.push(value),
        }
    }
}
