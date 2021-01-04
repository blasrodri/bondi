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

use ring::Ring;

use anyhow::Result;
pub struct Bondi<T> {
    ring: Ring<T>,
}

impl<T: Clone> Bondi<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: Ring::new(capacity),
        }
    }
    pub fn get_tx(&self) -> Result<()> {
        Ok(())
    }

    pub fn get_rx(&self) -> Result<()> {
        Ok(())
    }
}
