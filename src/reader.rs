use crate::ring::Ring;
use std::sync::Arc;

#[derive(Debug)]
pub struct Reader<T> {
    ring: Arc<Ring<T>>,
    idx: usize,
}

impl<T: Clone + std::fmt::Debug> Reader<T> {
    pub fn new(ring: Arc<Ring<T>>, idx: usize) -> Self {
        // TODO: signal the Ring that there is a new consumer
        // and accomodate its index right where the slowest consumer is
        Self { ring, idx: idx }
    }

    pub fn read(&self) -> T {
        let result = self.ring.get(self.idx);
        result.clone()
    }
}
