//! Program instruction processor for end-to-end testing and instruction counts

use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::instruction::PythClientInstruction;
use pyth_sdk_solana::state::load_price_account;

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = PythClientInstruction::try_from_slice(input).unwrap();
    match instruction {
        PythClientInstruction::Divide {
            numerator,
            denominator,
        } => {
            numerator.div(&denominator);
            Ok(())
        }
        PythClientInstruction::Multiply { x, y } => {
            x.mul(&y);
            Ok(())
        }
        PythClientInstruction::Add { x, y } => {
            x.add(&y);
            Ok(())
        }
        PythClientInstruction::Normalize { x } => {
            x.normalize();
            Ok(())
        }
        PythClientInstruction::ScaleToExponent { x, expo } => {
            x.scale_to_exponent(expo);
            Ok(())
        }
        PythClientInstruction::Noop => Ok(()),
        PythClientInstruction::PriceStatusCheck {
            price_account_data,
            expected_price_status,
        } => {
            let price_account = load_price_account(price_account_data.as_ref())?;
            let price = price_account.to_price_feed(&Pubkey::default());

            if price.status == expected_price_status {
                Ok(())
            } else {
                Err(ProgramError::Custom(0))
            }
        }
    }
}
