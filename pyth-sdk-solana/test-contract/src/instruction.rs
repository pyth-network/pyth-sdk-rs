//! Program instructions for end-to-end testing and instruction counts

use bytemuck::bytes_of;

use pyth_sdk_solana::state::PriceAccount;
use pyth_sdk_solana::{
    Price,
    PriceStatus,
};

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

    PriceStatusCheck {
        // A Price serialized as a vector of bytes. This field is stored as a vector of bytes
        // (instead of a Price) so that we do not have to add Borsh serialization to all
        // structs, which is expensive.
        price_account_data:    Vec<u8>,
        expected_price_status: PriceStatus,
    },
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

// Returns ok if price account status matches given expected price status.
pub fn price_status_check(price: &PriceAccount, expected_price_status: PriceStatus) -> Instruction {
    Instruction {
        program_id: id(),
        accounts:   vec![],
        data:       PythClientInstruction::PriceStatusCheck {
            price_account_data: bytes_of(price).to_vec(),
            expected_price_status,
        }
        .try_to_vec()
        .unwrap(),
    }
}
