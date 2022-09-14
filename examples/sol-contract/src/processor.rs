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
    let instruction = PythClientInstruction::try_from_slice(input).unwrap();
    match instruction {
        PythClientInstruction::Loan2Value {} => {
            let loan = &_accounts[0];
            let collateral = &_accounts[1];
            msg!("The loan key is {}.", loan.key);
            msg!("The collateral key is {}.", collateral.key);

            msg!("Assume 1 unit of loan and 3000 unit of collateral.");
            let loan_cnt = 1;
            let collateral_cnt = 3000;
            
            let feed1 = load_price_feed_from_account_info(&loan).unwrap();
            let result1 = feed1.get_current_price().unwrap();
            let loan_value = result1.price * loan_cnt;

            let feed2 = load_price_feed_from_account_info(&collateral).unwrap();
            let result2 = feed2.get_current_price().unwrap();
            let collateral_value = result2.price * collateral_cnt;

            if collateral_value > loan_value {
                msg!("Loan unit price is {}.", result1.price);
                msg!("Collateral unit price is {}.", result2.price);
                msg!("The value of collateral is higher.");
                Ok(())
            } else {
                msg!("The value of loan is higher!");
                Err(ProgramError::Custom(0))
            }
        }
    }
}
