use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by Pyth.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PythError {
    // 0
    /// Invalid account data -- either insufficient data, or incorrect magic number
    #[error("Failed to convert account into a Pyth account")]
    InvalidAccountData,
    /// Wrong version number
    #[error("Incorrect version number for Pyth account")]
    BadVersionNumber,
    /// Tried reading an account with the wrong type, e.g., tried to read
    /// a price account as a product account.
    #[error("Incorrect account type")]
    WrongAccountType,
}

impl From<PythError> for ProgramError {
    fn from(e: PythError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
