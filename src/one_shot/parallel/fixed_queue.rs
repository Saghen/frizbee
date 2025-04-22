use std::{
    alloc::{alloc, dealloc, Layout},
    cell::RefCell,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ThreadBatch {
    pub offset: usize,
    pub pos: usize,
}

thread_local!(static THREAD_BATCH: RefCell<Option<ThreadBatch>> = RefCell::new(None));

/// Thread-safe append-only fixed-size queue for parallel matching in cases where the number of
/// elements is known ahead of time. !!! Only one of these can exist at a time per thread !!!
#[derive(Debug)]
pub struct FixedBatchedQueue<T> {
    consumed: bool,
    layout: Layout,
    data: *mut T,
    len: usize,
    /// Number of slots to commit at once for a thread
    batch_size: usize,
    /// Current batch index
    batch_idx: AtomicUsize,
}

unsafe impl<T: Send> Send for FixedBatchedQueue<T> {}
unsafe impl<T: Send> Sync for FixedBatchedQueue<T> {}

impl<T> FixedBatchedQueue<T> {
    pub fn new(capacity: usize, batch_size: usize, thread_count: usize) -> Self {
        assert!(batch_size > 0, "Batch size must be greater than 0");
        assert!(
            capacity >= batch_size,
            "Capacity must be greater or equal to batch size"
        );
        // Round up the capacity to the next multiple of the batch size
        // Include a buffer of `thread_count * batch_size`
        let capacity = (thread_count + capacity.div_ceil(batch_size)) * batch_size;
        let layout = Layout::array::<T>(capacity).expect("Overflow cannot happen");
        FixedBatchedQueue {
            consumed: false,
            layout,
            data: unsafe { alloc(layout) } as *mut T,
            len: capacity,
            batch_size,
            batch_idx: AtomicUsize::new(0),
        }
    }

    // Pushes a single element to the current batch, creating a new thread-local batch if needed
    pub fn push(&self, value: T) {
        THREAD_BATCH.with(|b| {
            let mut batch = b.borrow_mut();
            // Allocate a new batch if needed
            match batch.as_ref() {
                None => {
                    batch.replace(self.alloc_batch());
                }
                Some(b) if b.pos == self.batch_size => {
                    batch.replace(self.alloc_batch());
                }
                _ => {}
            }

            // Write the value to the current batch
            let batch = batch.as_mut().unwrap();
            unsafe {
                let ptr = self.data.add(batch.pos + batch.offset);
                ptr.write(value);
            }
            batch.pos += 1;
        });
    }

    fn alloc_batch(&self) -> ThreadBatch {
        loop {
            let current_batch_idx = self.batch_idx.load(Ordering::Relaxed);
            match self.batch_idx.compare_exchange(
                current_batch_idx,
                current_batch_idx + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                // Successfully reserved the batch [current_len, required_len)
                Ok(_) => {
                    let offset = current_batch_idx * self.batch_size;
                    if offset + self.batch_size > self.len {
                        panic!("BatchedLockFreeQueue overflow");
                    }
                    return ThreadBatch { offset, pos: 0 };
                }
                // Contention: another thread succeeded. Retry loop.
                Err(_) => continue,
            }
        }
    }

    /// Consumes the vector and converts it into `Vec<T>`. We must clean up the dangling
    /// commited slots with no elements
    pub fn into_vec(mut self) -> Vec<T> {
        assert!(
            !self.consumed,
            "Cannot consume a consumed BatchedLockFreeQueue"
        );
        self.consumed = true;
        // TODO: clean up dangling committed slots
        unsafe { Vec::from_raw_parts(self.data, 1, self.len) }
    }
}

impl<T> Drop for FixedBatchedQueue<T> {
    fn drop(&mut self) {
        if !self.consumed {
            unsafe { dealloc(self.data as *mut u8, self.layout) };
        }
    }
}
