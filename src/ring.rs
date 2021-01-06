use anyhow::Result;
use std::{
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::errors::BondiError;

const MAX_CONSUMERS: usize = 100;
#[derive(Debug)]
pub struct Ring<T> {
    capacity: usize,
    buffer: Vec<T>,
    writer_idx: AtomicUsize,
    consumer_idx: [usize; MAX_CONSUMERS],
    num_consumers: AtomicUsize,
}

impl<T: Clone + std::fmt::Debug> Ring<T> {
    pub fn new(capacity: usize) -> Self {
        Ring {
            capacity,
            buffer: Vec::with_capacity(capacity),
            writer_idx: AtomicUsize::new(0),
            consumer_idx: [0; MAX_CONSUMERS],
            num_consumers: AtomicUsize::new(0),
        }
    }

    pub fn new_consumer(&self) -> Result<usize> {
        let num_consumers = self.num_consumers.fetch_add(1, Ordering::SeqCst);
        if num_consumers >= self.consumer_idx.len() {
            return Err(BondiError::InvalidInput.into());
        }
        Ok(num_consumers)
    }

    pub fn get_slowest_reader(&self) -> Result<usize> {
        if self.num_consumers.load(Ordering::SeqCst) == 0 {
            return Err(BondiError::NoReaderAvailable.into());
        }
        Ok(*self
            .consumer_idx
            .iter()
            .take(self.num_consumers.load(Ordering::SeqCst))
            .min()
            .unwrap())
    }

    pub fn insert(&self, item: T) {
        let writer_idx = self.writer_idx.load(Ordering::SeqCst) % self.capacity;
        unsafe {
            let buff = (self.buffer.as_ptr()).offset(writer_idx as isize) as _;
            ptr::write(buff, item);
        };
        self.writer_idx
            .store((writer_idx + 1) % self.capacity, Ordering::SeqCst);
    }

    pub fn get(&self, idx: usize) -> T {
        let consumer_idx = self.consumer_idx[idx];
        while self.writer_idx.load(Ordering::SeqCst) == consumer_idx {
            // TODO: get rid of the wait. Ideally subscribe a thread park
            // that will be unparked whenever the write advances.
            // but I'm not sure how to do it using lock-free primitives only. Considered
            // using condvars, but didn't like having to Arc<Mutex>...

            let duration = std::time::Duration::from_micros(10);
            std::thread::sleep(duration);
        }
        unsafe {
            let buff = (self.buffer.as_ptr()).offset((consumer_idx % self.capacity) as isize);
            let consumer_arr = self.consumer_idx.as_ptr().offset(idx as isize) as *mut usize;
            consumer_arr.write(consumer_idx + 1);
            ptr::read(buff).clone()
        }
    }
}
