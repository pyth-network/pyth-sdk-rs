//! Program instructions for end-to-end testing and instruction counts

use pyth_sdk_solana::Price;

use crate::id;
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use solana_program::instruction::Instruction;

/// Instructions supported by the pyth-client program, used for testing and
/// instruction counts
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum PythClientInstruction {
    Divide {
        numerator:   Price,
        denominator: Price,
    },
    Multiply {
        x: Price,
        y: Price,
    },
    Add {
        x: Price,
        y: Price,
    },
    ScaleToExponent {
        x:    Price,
        expo: i32,
    },
    Normalize {
        x: Price,
    },
    /// Don't do anything for comparison
    ///
    /// No accounts required for this instruction
    Noop,
}

pub fn divide(numerator: Price, denominator: Price) -> Instruction {
    Instruction {
        program_id: id(),
        accounts:   vec![],
        data:       PythClientInstruction::Divide {
            numerator,
            denominator,
        }
        .try_to_vec()
        .unwrap(),
    }
}

pub fn multiply(x: Price, y: Price) -> Instruction {
    Instruction {
        program_id: id(),
        accounts:   vec![],
        data:       PythClientInstruction::Multiply { x, y }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn add(x: Price, y: Price) -> Instruction {
    Instruction {
        program_id: id(),
        accounts:   vec![],
        data:       PythClientInstruction::Add { x, y }.try_to_vec().unwrap(),
    }
}

pub fn scale_to_exponent(x: Price, expo: i32) -> Instruction {
    Instruction {
        program_id: id(),
        accounts:   vec![],
        data:       PythClientInstruction::ScaleToExponent { x, expo }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn normalize(x: Price) -> Instruction {
    Instruction {
        program_id: id(),
        accounts:   vec![],
        data:       PythClientInstruction::Normalize { x }.try_to_vec().unwrap(),
    }
}

/// Noop instruction for comparison purposes
pub fn noop() -> Instruction {
    Instruction {
        program_id: id(),
        accounts:   vec![],
        data:       PythClientInstruction::Noop.try_to_vec().unwrap(),
    }
}
