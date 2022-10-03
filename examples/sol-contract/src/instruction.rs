//! Program instructions

use borsh::BorshSerialize;
use borsh::BorshDeserialize;

// A solana program contains a number of instructions.
// And this example contract contains only one instruction.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum PythClientInstruction {
    Loan2Value {}, // in this enum, Loan2Value is number 0
    Init{},        // and Init is 1
}
