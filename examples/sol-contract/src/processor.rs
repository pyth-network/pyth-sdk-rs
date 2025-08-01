//! Program instruction processor
//! Only the program admin can issue the Init instruction.
//! And anyone can check the loan with the Loan2Value instruction.

use std::cmp::Ordering;

use solana_program::account_info::{
    next_account_info,
    AccountInfo,
};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memcpy;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::clock::Clock;
use solana_program::sysvar::Sysvar;

use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use pyth_sdk_solana::state::SolanaPriceAccount;

use crate::instruction::ExampleInstructions;
use crate::state::AdminConfig;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let signer = next_account_info(account_iter)?;
    let admin_config_account = next_account_info(account_iter)?;
    let pyth_loan_account = next_account_info(account_iter)?;
    let pyth_collateral_account = next_account_info(account_iter)?;

    let instruction = ExampleInstructions::try_from_slice(input)?;
    match instruction {
        ExampleInstructions::Init {} => {
            // Only an authorized key should be able to configure the price feed id for each asset
            if !(signer.key == program_id && signer.is_signer) {
                return Err(ProgramError::Custom(0));
            }

            let mut config = AdminConfig::try_from_slice(&admin_config_account.try_borrow_data()?)?;

            if config.is_initialized {
                return Err(ProgramError::Custom(1));
            }

            config.is_initialized = true;
            config.loan_price_feed_id = *pyth_loan_account.key;
            config.collateral_price_feed_id = *pyth_collateral_account.key;

            // Make sure these Pyth price accounts can be loaded
            SolanaPriceAccount::account_info_to_feed(pyth_loan_account)?;
            SolanaPriceAccount::account_info_to_feed(pyth_collateral_account)?;

            let config_data = config.try_to_vec()?;
            let config_dst = &mut admin_config_account.try_borrow_mut_data()?;
            sol_memcpy(config_dst, &config_data, 1 + 32 + 32);
            Ok(())
        }

        ExampleInstructions::Loan2Value {
            loan_qty,
            collateral_qty,
        } => {
            msg!("Loan quantity is {}.", loan_qty);
            msg!("Collateral quantity is {}.", collateral_qty);

            let config = AdminConfig::try_from_slice(&admin_config_account.try_borrow_data()?)?;

            if !config.is_initialized {
                return Err(ProgramError::Custom(1));
            }

            if config.loan_price_feed_id != *pyth_loan_account.key
                || config.collateral_price_feed_id != *pyth_collateral_account.key
            {
                return Err(ProgramError::Custom(2));
            }

            // With high confidence, the maximum value of the loan is
            // (price + conf) * loan_qty * 10 ^ (expo).
            // Here is more explanation on confidence interval in Pyth:
            // https://docs.pyth.network/consume-data/best-practices
            let feed1 = SolanaPriceAccount::account_info_to_feed(pyth_loan_account)?;
            let current_timestamp1 = Clock::get()?.unix_timestamp;
            let result1 = feed1
                .get_price_no_older_than(current_timestamp1, 60)
                .ok_or(ProgramError::Custom(3))?;
            let loan_max_price = result1
                .price
                .checked_add(result1.conf as i64)
                .ok_or(ProgramError::Custom(4))?;
            let mut loan_max_value = loan_max_price
                .checked_mul(loan_qty)
                .ok_or(ProgramError::Custom(4))?;
            msg!(
                "The maximum loan value is {} * 10^({}).",
                loan_max_value,
                result1.expo
            );

            // With high confidence, the minimum value of the collateral is
            // (price - conf) * collateral_qty * 10 ^ (expo).
            // Here is more explanation on confidence interval in Pyth:
            // https://docs.pyth.network/consume-data/best-practices
            let feed2 = SolanaPriceAccount::account_info_to_feed(pyth_collateral_account)?;
            let current_timestamp2 = Clock::get()?.unix_timestamp;
            let result2 = feed2
                .get_price_no_older_than(current_timestamp2, 60)
                .ok_or(ProgramError::Custom(3))?;
            let collateral_min_price = result2
                .price
                .checked_sub(result2.conf as i64)
                .ok_or(ProgramError::Custom(4))?;
            let mut collateral_min_value = collateral_min_price
                .checked_mul(collateral_qty)
                .ok_or(ProgramError::Custom(4))?;
            msg!(
                "The minimum collateral value is {} * 10^({}).",
                collateral_min_value,
                result2.expo
            );

            // If the loan and collateral prices use different exponent,
            // normalize the value.
            match result1.expo.cmp(&result2.expo) {
                Ordering::Greater => {
                    let normalize = 10_i64
                        .checked_pow((result1.expo - result2.expo) as u32)
                        .ok_or(ProgramError::Custom(4))?;
                    collateral_min_value = collateral_min_value
                        .checked_mul(normalize)
                        .ok_or(ProgramError::Custom(4))?;
                }
                Ordering::Less => {
                    let normalize = 10_i64
                        .checked_pow((result2.expo - result1.expo) as u32)
                        .ok_or(ProgramError::Custom(4))?;
                    loan_max_value = loan_max_value
                        .checked_mul(normalize)
                        .ok_or(ProgramError::Custom(4))?;
                }
                Ordering::Equal => {}
            }
            // Check whether the value of the collateral is higher.
            if collateral_min_value > loan_max_value {
                msg!("The value of the collateral is higher.");
                Ok(())
            } else {
                msg!("The value of the loan is higher!");
                Err(ProgramError::Custom(5))
            }
        }
    }
}
