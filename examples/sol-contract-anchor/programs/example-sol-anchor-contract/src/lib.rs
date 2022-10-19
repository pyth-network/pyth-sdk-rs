use std::mem::size_of;
use anchor_lang::prelude::*;
use solana_program::account_info::AccountInfo;
use pyth_sdk_solana::load_price_feed_from_account_info;

declare_id!("GZwQNVL14fd7ubdVvucnE7aYytmtiPWLJ3638y31hBAL");

#[account]
pub struct AdminConfig {
    pub is_initialized:           bool,
    pub loan_price_feed_id:       Pubkey,
    pub collateral_price_feed_id: Pubkey,
}

#[derive(Accounts)]
pub struct InitRequest<'info> {
    #[account(address = *program_id)]
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
    /// CHECK: Pyth structs don't seem to support Anchor deserialization
    #[account(address = config.loan_price_feed_id @ ErrorCode::InvalidArgument)]
    pub pyth_loan_account: AccountInfo<'info>,
    /// CHECK: Pyth structs don't seem to support Anchor deserialization
    #[account(address = config.collateral_price_feed_id @ ErrorCode::InvalidArgument)]
    pub pyth_collateral_account: AccountInfo<'info>,
}

#[program]
pub mod example_sol_anchor_contract {
    use super::*;

    pub fn init(ctx: Context<InitRequest>, config: AdminConfig) -> Result<()> {
        if ctx.accounts.config.is_initialized {
            return Err(error!(ErrorCode::ReInitialize));
        }
        ctx.accounts.config.set_inner(config);
        Ok(())
    }

    pub fn loan_to_value(ctx: Context<QueryRequest>, loan_qty: i64, collateral_qty: i64) -> Result<()> {
        if !ctx.accounts.config.is_initialized {
            return Err(error!(ErrorCode::UnInitialize));
        }

        msg!("Loan quantity is {}.", loan_qty);
        msg!("Collateral quantity is {}.", collateral_qty);

        let pyth_loan_account = &ctx.accounts.pyth_loan_account;
        let pyth_collateral_account = &ctx.accounts.pyth_collateral_account;
        // With high confidence, the maximum value of the loan is
        // (price + conf) * loan_qty * 10 ^ (expo).
        // Here is more explanation on confidence interval in Pyth:
        // https://docs.pyth.network/consume-data/best-practices
        let feed1 = load_price_feed_from_account_info(pyth_loan_account)
            .map_err(|_x| error!(ErrorCode::PythError))?;
        let result1 = feed1.get_current_price()
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
        let feed2 = load_price_feed_from_account_info(pyth_collateral_account)
                        .map_err(|_x| error!(ErrorCode::PythError))?;
        let result2 = feed2.get_current_price()
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

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("The config has already been initialized.")]
    ReInitialize,
    #[msg("The config has not been initialized.")]
    UnInitialize,
    #[msg("Argument is invalid.")]
    InvalidArgument,
    #[msg("An overflow occurs.")]
    Overflow,
    #[msg("Pyth has an internal error.")]
    PythError,
    #[msg("Pyth price oracle is offline.")]
    PythOffline,
    #[msg("The loan value is higher than the collateral value.")]
    LoanValueTooHigh,
}
