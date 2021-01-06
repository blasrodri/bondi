use thiserror::Error;

#[derive(Debug, Error)]
pub enum BondiError {
    #[error("Invalid Input")]
    InvalidInput,
    #[error("Writer already exists")]
    WriterAlreadyExists,
    #[error("No reader available")]
    NoReaderAvailable,
}
