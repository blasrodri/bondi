//! # Bondi
//!
//! This is built to provide an inter-process mechanism to communicate
//! between different parties.
//!
//! It allows a [Writer](writer::Writer) send a message that can be read
//! by multiple [Reader](reader::Reader) s concurrently. The role of `Bondi` is to sync these operations,
//! while keeping things **fast**.
pub mod errors;
pub mod reader;
pub mod ring;
pub mod writer;

use reader::Reader;
use ring::Ring;

use anyhow::Result;

use std::sync::Arc;
pub struct Bondi<T> {
    ring: Arc<Ring<T>>,
}

impl<T: Clone> Bondi<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: Arc::new(Ring::new(capacity)),
        }
    }
    pub fn get_tx(&self) -> Result<()> {
        Ok(())
    }

    pub fn get_rx(&self) -> Result<Reader<T>> {
        Ok(Reader::new(Arc::clone(&self.ring)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_a_reader() {
        let bondi = Bondi::<usize>::new(100);
        let mut reader = bondi.get_rx().unwrap();
        std::thread::spawn(move || {
            let _ = reader.read();
        });
        // there is nothing to read, so don't join
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
