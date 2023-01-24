use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
pub enum OracleError {
    #[error("initial endpoint should not be greater than or equal to final endpoint")]
    InitialEndpointExceedsFinalEndpoint,
    #[error("initial discount should not exceed final discount, for collateral valuation")]
    InitialDiscountExceedsFinalDiscount,
    #[error("final discount rate should not be greater than the discount precision")]
    FinalDiscountExceedsPrecision,
    #[error("initial premium should not exceed final premium, for borrow valuation")]
    InitialPremiumExceedsFinalPremium,
    #[error("None encountered")]
    NoneEncountered,
    #[error("i64 try from error")]
    I64ConversionError,
}