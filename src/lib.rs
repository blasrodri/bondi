//! # Bondi
//!
//! This is built to provide an inter-process mechanism to communicate
//! between different parties.
//!
//! It allows a [Writer](writer::Writer) send a message that can be read
//! by multiple [Reader](reader::Reader) s concurrently. The role of `Bondi` is to sync these operations,
//! while keeping things **fast**.
//!
//! ### A Simple example
//! ```
//! // initialize a writer and two readers
//! // send 100 `Message`s, and receive them from different threads
//! struct Message(usize)
//!
//! fn main() {
//!     let bondi = Bondi::<Message>::new(100);
//!     let writer = bondi.get_tx().unwrap();
//!     let reader = bondi.get_rx().unwrap();
//!     let reader2 = bondi.get_rx().unwrap();
//!
//!     std::thread::spawn(move || {
//!         for i in 0..100 {
//!             writer.write(Message(i));
//!         }
//!     });
//!
//!     std::thread::spawn(move || {
//!         for i in 0..100 {
//!             reader.read().unwrap();
//!         }
//!     });
//!
//!     std::thread::spawn(move || {
//!         for i in 0..100 {
//!             reader2.read().unwrap();
//!         }
//!     }).join().unwrap();
//! }
//! ```

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
    fn write_and_read_something_one_reader_no_wrap() {
        let bondi = Bondi::<usize>::new(100);
        let mut writer = bondi.get_tx().unwrap();
        let reader = bondi.get_rx().unwrap();
        std::thread::spawn(move || {
            for i in 0..100 {
                writer.write(i);
            }
        });

        std::thread::spawn(move || {
            for i in 0..100 {
                assert_eq!(reader.read(), i);
            }
        })
        .join()
        .unwrap();
    }

    #[test]
    fn write_and_read_something_two_readers_no_wrap() {
        let bondi = Bondi::<usize>::new(100);
        let mut writer = bondi.get_tx().unwrap();
        let reader = bondi.get_rx().unwrap();
        let reader2 = bondi.get_rx().unwrap();

        std::thread::spawn(move || {
            for i in 0..100 {
                writer.write(i);
                assert_eq!(reader.read(), i);
            }
        });

        std::thread::spawn(move || {
            for i in 0..100 {
                assert_eq!(reader2.read(), i);
                // dbg!(i);
            }
        })
        .join()
        .unwrap();
    }

    #[test]
    fn write_and_read_something_many_readers_with_wrapping() {
        let bondi = Bondi::<usize>::new(100);
        let mut writer = bondi.get_tx().unwrap();
        let readers = (0..99).into_iter().map(|_| bondi.get_rx().unwrap());
        let last_reader = bondi.get_rx().unwrap();
        std::thread::spawn(move || {
            for i in 0..200 {
                writer.write(i);
            }
        });

        for reader in readers {
            std::thread::spawn(move || {
                for i in 0..200 {
                    assert_eq!(reader.read(), i);
                }
            });
        }

        std::thread::spawn(move || {
            for i in 0..200 {
                assert_eq!(last_reader.read(), i);
            }
        })
        .join()
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "No reader available")]
    fn too_many_readers() {
        let max_readers_available = 1000;
        let bondi = Bondi::<usize>::new(max_readers_available);
        let _readers = (0..max_readers_available)
            .into_iter()
            .map(|_| bondi.get_rx().unwrap())
            .collect::<Vec<_>>();
    }
}
