use crate::ring::Ring;
use std::sync::Arc;
pub struct Writer<T> {
    ring: Arc<Ring<T>>,
}

impl<T: Clone + std::fmt::Debug> Writer<T> {
    pub fn new(ring: Arc<Ring<T>>) -> Self {
        Self { ring }
    }
    pub fn write(&self, item: T) {
        self.ring.insert(item)
    }
}
