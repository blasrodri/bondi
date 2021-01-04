use crate::ring::Ring;
use std::sync::Arc;

pub struct Reader<T> {
    ring: Arc<Ring<T>>,
    idx: usize,
}

impl<T: Clone> Reader<T> {
    pub fn new(ring: Arc<Ring<T>>) -> Self {
        Self { ring, idx: 0 }
    }

    pub fn read(&mut self) -> T {
        let result = self.ring.get(self.idx);
        self.idx += 1;
        result.clone()
    }
}
