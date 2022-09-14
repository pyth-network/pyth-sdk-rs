//! Program instructions

use crate::id;
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use solana_program::instruction::Instruction;
use solana_program::instruction::AccountMeta;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum PythClientInstruction {
    Loan2Value {},
}

pub fn loan_to_value(loan: AccountMeta, collateral: AccountMeta) -> Instruction {
    Instruction {
        program_id: id(),
        accounts:   vec![loan, collateral],
        data:       PythClientInstruction::Loan2Value {}
        .try_to_vec()
        .unwrap(),
    }
}
