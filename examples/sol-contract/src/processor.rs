//! Program instruction processor
//! Only the program admin can issue the Init instruction.
//! And anyone can check the loan with the Loan2Value instruction.

use solana_program::account_info::{
    next_account_info,
    AccountInfo,
};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{
    IsInitialized,
    Pack,
};
use solana_program::pubkey::Pubkey;

use borsh::BorshDeserialize;
use pyth_sdk_solana::load_price_feed_from_account_info;

use crate::instruction::ExampleInstructions;
use crate::state::AdminConfig;

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let account_iter = &mut _accounts.iter();
    let signer = next_account_info(account_iter)?;
    let data_account = next_account_info(account_iter)?;
    let pyth_loan_account = next_account_info(account_iter)?;
    let pyth_collateral_account = next_account_info(account_iter)?;

    let instruction = ExampleInstructions::try_from_slice(input)?;
    match instruction {
        ExampleInstructions::Init {} => {
            if !(signer.key == _program_id && signer.is_signer) {
                return Err(ProgramError::Custom(0));
            }

            let mut loan_info = AdminConfig::unpack_from_slice(&data_account.try_borrow_data()?)?;

            if loan_info.is_initialized() {
                return Err(ProgramError::Custom(1));
            }

            loan_info.is_initialized = true;
            loan_info.loan_price_feed_id = *pyth_loan_account.key;
            loan_info.collateral_price_feed_id = *pyth_collateral_account.key;

            AdminConfig::pack(loan_info, &mut data_account.try_borrow_mut_data()?)?;
            Ok(())
        }
        ExampleInstructions::Loan2Value {
            loan_qty,
            collateral_qty,
        } => {
            msg!("Loan quantity is {}.", loan_qty);
            msg!("Collateral quantity is {}.", collateral_qty);

            let loan_info = AdminConfig::unpack_from_slice(&data_account.try_borrow_data()?)?;

            if !loan_info.is_initialized() {
                return Err(ProgramError::Custom(1));
            }

            if loan_info.loan_price_feed_id != *pyth_loan_account.key
                || loan_info.collateral_price_feed_id != *pyth_collateral_account.key
            {
                return Err(ProgramError::Custom(2));
            }

            // Calculate the maximum value of the loan using Pyth.
            let feed1 = load_price_feed_from_account_info(pyth_loan_account)?;
            let result1 = feed1.get_current_price().ok_or(ProgramError::Custom(3))?;
            let loan_max_price = result1
                .price
                .checked_add(result1.conf as i64)
                .ok_or(ProgramError::Custom(4))?;
            let loan_max_value = loan_max_price
                .checked_mul(loan_qty)
                .ok_or(ProgramError::Custom(4))?;

            // Calculate the minimum value of the collateral using Pyth.
            let feed2 = load_price_feed_from_account_info(pyth_collateral_account)?;
            let result2 = feed2.get_current_price().ok_or(ProgramError::Custom(3))?;
            let collateral_min_price = result2
                .price
                .checked_sub(result2.conf as i64)
                .ok_or(ProgramError::Custom(4))?;
            let collateral_min_value = collateral_min_price
                .checked_mul(collateral_qty)
                .ok_or(ProgramError::Custom(4))?;

            // Check whether the value of the collateral is higher.
            msg!(
                "The maximum loan value is {} * 10^({}).",
                loan_max_value,
                result1.expo
            );
            msg!(
                "The minimum collateral value is {} * 10^({}).",
                collateral_min_value,
                result2.expo
            );

            if collateral_min_value > loan_max_value {
                msg!("The value of the collateral is higher.");
                return Ok(());
            } else {
                msg!("The value of the loan is higher!");
                return Err(ProgramError::Custom(5));
            }
        }
    }
}
