//! Program instructions

use borsh::BorshSerialize;
use borsh::BorshDeserialize;

// A solana program contains a number of instructions.
// And this example contract contains only one instruction.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum PythClientInstruction {
    Loan2Value {}, // In this enum, Loan2Value is number 0.
}
