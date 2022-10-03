//! Program instruction processor


use solana_program::msg;
use solana_program::pubkey::Pubkey;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack};
use solana_program::account_info::{next_account_info, AccountInfo};

use borsh::BorshDeserialize;
use pyth_sdk_solana::load_price_feed_from_account_info;

use crate::state::LoanInfo;
use crate::instruction::PythClientInstruction;

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

    let instruction = PythClientInstruction::try_from_slice(input)?;
    match instruction {
        PythClientInstruction::Init {} => {
            // Only the program admin can initialize a loan.
            if !(signer.key == _program_id && signer.is_signer) {
                return Err(ProgramError::Custom(1))
            }

            let mut loan_info = LoanInfo::unpack_from_slice(
                &data_account.try_borrow_data()?)?;

            if loan_info.is_initialized() {
                return Err(ProgramError::Custom(1))
            }

            loan_info.is_initialized = true;
            loan_info.loan_key = *pyth_loan_account.key;
            loan_info.collateral_key = *pyth_collateral_account.key;
            // Give some dummy numbers for simplicity of this example.
            loan_info.loan_qty = 1;
            loan_info.collateral_qty = 3000;

            msg!("The loan key is {}.", loan_info.loan_key);
            msg!("The collateral key is {}.", loan_info.collateral_key);
            msg!("Assume 1 unit of loan and 3000 unit of collateral.");
            LoanInfo::pack(loan_info, &mut data_account.try_borrow_mut_data()?)?;
            Ok(())
        },
        PythClientInstruction::Loan2Value {} => {
            // Anyone can check the loan to value ratio.
            let loan_info = LoanInfo::unpack_from_slice(
                &data_account.try_borrow_data()?)?;

            if !loan_info.is_initialized() {
                return Err(ProgramError::Custom(1))
            }

            if loan_info.loan_key != *pyth_loan_account.key ||
                loan_info.collateral_key != *pyth_collateral_account.key {
                    return Err(ProgramError::Custom(1))
                }

            // Calculate the value of the loan using Pyth price.
            let feed1 = load_price_feed_from_account_info(pyth_loan_account)?;
            let result1 = feed1.get_current_price()
                .ok_or(ProgramError::Custom(0))?;
            let loan_value = result1.price.checked_mul(loan_info.loan_qty)
                .ok_or(ProgramError::Custom(0))?;

            // Calculate the value of the loan using Pyth price.
            let feed2 = load_price_feed_from_account_info(pyth_collateral_account)?;
            let result2 = feed2.get_current_price()
                .ok_or(ProgramError::Custom(0))?;
            let collateral_value = result2.price.checked_mul(loan_info.collateral_qty)
                .ok_or(ProgramError::Custom(1))?;

            // Check whether the value of the collateral is higher.
            if collateral_value > loan_value {
                msg!("Loan unit price is {}.", result1.price);
                msg!("Collateral unit price is {}.", result2.price);
                msg!("The value of collateral is higher.");
                return Ok(())
            } else {
                msg!("The value of loan is higher!");
                return Err(ProgramError::Custom(2))
            }
        }
    }
}
