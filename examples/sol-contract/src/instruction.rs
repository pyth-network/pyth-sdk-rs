//! Program instructions
//! A solana program contains a number of instructions.
//! There are 2 instructions in this example:
//!     Init{} initializing some loan information and
//!     Loan2Value{} checking the loan-to-value ratio of the loan.

use borsh_derive::{
    BorshDeserialize,
    BorshSerialize,
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ExampleInstructions {
    Init {},
    Loan2Value {
        loan_qty:       i64,
        collateral_qty: i64,
    },
}
