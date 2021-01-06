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

use errors::BondiError;
use reader::Reader;
use ring::Ring;

use anyhow::Result;
use writer::Writer;

use std::{cell::Cell, sync::Arc};
pub struct Bondi<T> {
    ring: Arc<Ring<T>>,
    writer_created: Cell<bool>,
}

impl<T: Clone + std::fmt::Debug> Bondi<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: Arc::new(Ring::new(capacity)),
            writer_created: Cell::new(false),
        }
    }
    pub fn get_tx(&self) -> Result<Writer<T>> {
        if self.writer_created.get() {
            return Err(BondiError::WriterAlreadyExists.into());
        }

        self.writer_created.set(true);
        Ok(Writer::new(Arc::clone(&self.ring)))
    }

    pub fn get_rx(&self) -> Result<Reader<T>> {
        let num_consumers = self.ring.new_consumer()?;
        Ok(Reader::new(Arc::clone(&self.ring), num_consumers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg_attr(miri, ignore)]
    fn get_many_readers() {
        // smoke test to ensure that we can get multiple readers
        let bondi = Bondi::<usize>::new(100);
        let reader = bondi.get_rx().unwrap();
        let reader2 = bondi.get_rx().unwrap();
        std::thread::spawn(move || {
            let _ = reader.read();
        });

        std::thread::spawn(move || {
            let _ = reader2.read();
        });

        // there is nothing to read, so don't join
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    #[test]
    fn write_something() {
        let bondi = Bondi::<usize>::new(100);
        let mut writer = bondi.get_tx().unwrap();
        for i in 0..100 {
            writer.write(i);
        }
    }
    #[test]
    fn write_and_read_something() {
        let bondi = Bondi::<usize>::new(100);
        let mut writer = bondi.get_tx().unwrap();
        let reader = dbg!(bondi.get_rx().unwrap());
        let reader2 = bondi.get_rx().unwrap();
        for i in 0..100 {
            writer.write(i);
            assert_eq!(reader.read(), i);
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
        std::thread::spawn(move || {
            for i in 0..100 {
                dbg!(reader2.read(), i);
            }
        })
        .join()
        .unwrap();
    }
}
