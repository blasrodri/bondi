use thiserror::Error;

#[derive(Debug, Error)]
pub enum BondiError {
    #[error("Invalid Input")]
    InvalidInput,
}
