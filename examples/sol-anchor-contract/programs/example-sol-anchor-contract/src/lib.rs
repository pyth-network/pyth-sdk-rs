use std::mem::size_of;
use anchor_lang::prelude::*;
use solana_program::account_info::AccountInfo;

pub mod state;
use state::AdminConfig;
use state::PythPrice;

mod error;
use error::ErrorCode;

declare_id!("BZh3CP454Ca1C9yBp2tpGAkXoKFti9x8ShJLSxNDpoxa");

#[derive(Accounts)]
pub struct InitRequest<'info> {
    #[account(address = *program_id @ ErrorCode::Unauthorized)]
    pub program: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, space = 8 + size_of::<AdminConfig>())]
    pub config: Account<'info, AdminConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct QueryRequest<'info> {
    pub config: Account<'info, AdminConfig>,
    #[account(address = config.loan_price_feed_id @ ErrorCode::InvalidArgument)]
    pub pyth_loan_account: Account<'info, PythPrice>,
    #[account(address = config.collateral_price_feed_id @ ErrorCode::InvalidArgument)]
    pub pyth_collateral_account: Account<'info, PythPrice>,
}

#[program]
pub mod example_sol_anchor_contract {
    use super::*;

    pub fn init(ctx: Context<InitRequest>, config: AdminConfig) -> Result<()> {
        ctx.accounts.config.set_inner(config);
        Ok(())
    }

    pub fn loan_to_value(ctx: Context<QueryRequest>, loan_qty: i64, collateral_qty: i64) -> Result<()> {
        msg!("Loan quantity is {}.", loan_qty);
        msg!("Collateral quantity is {}.", collateral_qty);

        let loan_price = &ctx.accounts.pyth_loan_account.price;
        let collateral_price = &ctx.accounts.pyth_collateral_account.price;
        // With high confidence, the maximum value of the loan is
        // (price + conf) * loan_qty * 10 ^ (expo).
        // Here is more explanation on confidence interval in Pyth:
        // https://docs.pyth.network/consume-data/best-practices
        let result1 = loan_price
            .ok_or(ErrorCode::PythOffline)?;
        let loan_max_price = result1
            .price
            .checked_add(result1.conf as i64)
            .ok_or(ErrorCode::Overflow)?;
        let mut loan_max_value = loan_max_price
            .checked_mul(loan_qty)
            .ok_or(ErrorCode::Overflow)?;
        msg!(
            "The maximum loan value is {} * 10^({}).",
            loan_max_value,
            result1.expo
        );

        // With high confidence, the minimum value of the collateral is
        // (price - conf) * collateral_qty * 10 ^ (expo).
        // Here is more explanation on confidence interval in Pyth:
        // https://docs.pyth.network/consume-data/best-practices
        let result2 = collateral_price
            .ok_or(ErrorCode::PythOffline)?;
        let collateral_min_price = result2
            .price
            .checked_sub(result2.conf as i64)
            .ok_or(ErrorCode::Overflow)?;
        let mut collateral_min_value = collateral_min_price
            .checked_mul(collateral_qty)
            .ok_or(ErrorCode::Overflow)?;
        msg!(
            "The minimum collateral value is {} * 10^({}).",
            collateral_min_value,
            result2.expo
        );

        // If the loan and collateral prices use different exponent,
        // normalize the value.
        if result1.expo > result2.expo {
            let normalize = (10 as i64)
                .checked_pow((result1.expo - result2.expo) as u32)
                .ok_or(ErrorCode::Overflow)?;
            collateral_min_value = collateral_min_value
                .checked_mul(normalize)
                .ok_or(ErrorCode::Overflow)?;
        } else if result1.expo < result2.expo {
            let normalize = (10 as i64)
                .checked_pow((result2.expo - result1.expo) as u32)
                .ok_or(ErrorCode::Overflow)?;
            loan_max_value = loan_max_value
                .checked_mul(normalize)
                .ok_or(ErrorCode::Overflow)?;
        }

        // Check whether the value of the collateral is higher.
        if collateral_min_value > loan_max_value {
            msg!("The value of the collateral is higher.");
            return Ok(());
        } else {
            return Err(error!(ErrorCode::LoanValueTooHigh));
        }
    }
}
