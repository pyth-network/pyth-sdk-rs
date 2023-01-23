use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
pub enum LiquidityOracleError {
    #[error("deposits exceeds max depositable")]
    ExceedsMaxDeposits,
    #[error("initial discount rate should not be greater than final discount rate")]
    InitialDiscountExceedsFinalDiscount,
    #[error("final discount rate should not be greater than the discount precision")]
    FinalDiscountExceedsPrecision,
    #[error("None encountered")]
    NoneEncountered,
    #[error("i64 try from error")]
    I64ConversionError,
}