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
    consumer_idx: Vec<AtomicUsize>,
    num_consumers: AtomicUsize,
}

impl<T: Clone + std::fmt::Debug> Ring<T> {
    pub fn new(capacity: usize) -> Self {
        Ring {
            capacity,
            buffer: Vec::with_capacity(capacity),
            writer_idx: AtomicUsize::new(0),
            consumer_idx: (0..MAX_CONSUMERS)
                .into_iter()
                .map(|_| AtomicUsize::new(0))
                .collect(),
            num_consumers: AtomicUsize::new(0),
        }
    }

    pub fn new_consumer(&self) -> Result<usize> {
        if self.num_consumers.load(Ordering::SeqCst) == MAX_CONSUMERS {
            return Err(BondiError::NoReaderAvailable.into());
        }
        let num_consumers = self.num_consumers.fetch_add(1, Ordering::Relaxed);
        Ok(num_consumers)
    }

    pub fn get_slowest_reader(&self) -> Result<usize> {
        if self.num_consumers.load(Ordering::Relaxed) == 0 {
            return Err(BondiError::NoReaderAvailable.into());
        }

        Ok(self
            .consumer_idx
            .iter()
            .take(self.num_consumers.load(Ordering::Relaxed))
            .map(|x| x.load(Ordering::Relaxed))
            .min()
            .unwrap_or(0))
    }

    pub fn insert(&self, item: T) {
        if self.num_consumers.load(Ordering::Relaxed) > 0 {
            let mut slowest_reader = self.get_slowest_reader().unwrap();
            let mut writer_idx = self.writer_idx.load(Ordering::Relaxed);
            // only wait if there is a slowest reader that has not yet read in the slot
            // where the writer will insert the data.
            while writer_idx > 0
                && writer_idx % self.capacity == slowest_reader
                && writer_idx > slowest_reader
            {
                std::thread::park_timeout(std::time::Duration::from_micros(1));
                writer_idx = self.writer_idx.load(Ordering::Relaxed);
                slowest_reader = self.get_slowest_reader().unwrap();
            }
        }
        let writer_idx = self.writer_idx.load(Ordering::Relaxed) % self.capacity;
        unsafe {
            let buff = (self.buffer.as_ptr()).offset(writer_idx as isize) as _;
            ptr::write(buff, item);
        };
        self.writer_idx.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get(&self, idx: usize) -> T {
        let consumer_idx = self.consumer_idx[idx].load(Ordering::Relaxed);
        while self.writer_idx.load(Ordering::Relaxed) == consumer_idx {
            // TODO: get rid of the wait. Ideally subscribe a thread park
            // that will be unparked whenever the write advances.
            // but I'm not sure how to do it using lock-free primitives only. Considered
            // using condvars, but didn't like having to Arc<Mutex>...

            std::thread::park_timeout(std::time::Duration::from_micros(1));
        }
        let result = unsafe {
            let buff = (self.buffer.as_ptr()).offset((consumer_idx % self.capacity) as isize);
            ptr::read(buff).clone()
        };
        self.consumer_idx[idx].fetch_add(1, Ordering::Relaxed);
        result
    }
}
