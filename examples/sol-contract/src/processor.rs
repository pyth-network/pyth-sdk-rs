//! Program instruction processor

use solana_program::msg;
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;

use borsh::BorshDeserialize;
use crate::instruction::PythClientInstruction;
use pyth_sdk_solana::load_price_feed_from_account_info;

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    // Checking the validity of parameters is important.
    // This checking step is skipped in this example contract.
    let instruction = PythClientInstruction::try_from_slice(input);
    match instruction.unwrap() {
        PythClientInstruction::Loan2Value {} => {
            let loan = &_accounts[0];
            let collateral = &_accounts[1];
            msg!("The loan key is {}.", loan.key);
            msg!("The collateral key is {}.", collateral.key);
            msg!("Assume 1 unit of loan and 3000 unit of collateral.");
            let loan_cnt = 1;
            let collateral_cnt = 3000;

            // Calculate the value of the loan
            // Pyth is called to get the unit price of the loan.
            let loan_value;
            let feed1 = load_price_feed_from_account_info(&loan);
            let result1 = feed1.unwrap().get_current_price().unwrap();
            if let Some(v) = result1.price.checked_mul(loan_cnt) {
                loan_value = v;
            } else {
                // An overflow occurs for result1.price * loan_cnt.
                return Err(ProgramError::Custom(0))
            }

            // Calculate the value of the collateral
            // Pyth is called to get the unit price of the collateral.
            let collateral_value;
            let feed2 = load_price_feed_from_account_info(&collateral);
            let result2 = feed2.unwrap().get_current_price().unwrap();
            if let Some(v) = result2.price.checked_mul(collateral_cnt) {
                collateral_value = v;
            } else {
                // An overflow occurs for result2.price * collateral_cnt.
                return Err(ProgramError::Custom(0))
            }

            // Check whether the value of the collateral is higher.
            if collateral_value > loan_value {
                msg!("Loan unit price is {}.", result1.price);
                msg!("Collateral unit price is {}.", result2.price);
                msg!("The value of collateral is higher.");
                return Ok(())
            } else {
                msg!("The value of loan is higher!");
                return Err(ProgramError::Custom(1))
            }
        }
    }
}
