use anchor_lang::prelude::*;

pub mod state;
use state::AdminConfig;
use state::PriceFeed;

mod error;
use error::ErrorCode;

declare_id!("GFPM2LncpbWiLkePLs3QjcLVPw31B2h23FwFfhig79fh");

const BASE : f64 = 10.0;

#[program]
pub mod sol_anchor_contract {
    use super::*;

    pub fn init(ctx: Context<InitRequest>, config: AdminConfig) -> Result<()> {
        ctx.accounts.config.set_inner(config);
        Ok(())
    }

    pub fn loan_to_value(
        ctx: Context<QueryRequest>,
        loan_qty: i64,
        collateral_qty: i64,
    ) -> Result<()> {
        msg!("Loan quantity is {}.", loan_qty);
        msg!("Collateral quantity is {}.", collateral_qty);

        let loan_feed = &ctx.accounts.pyth_loan_account;
        let collateral_feed = &ctx.accounts.pyth_collateral_account;
        // With high confidence, the maximum value of the loan is
        // (price + conf) * loan_qty * 10 ^ (expo).
        // Here is more explanation on confidence interval in Pyth:
        // https://docs.pyth.network/consume-data/best-practices
        let current_timestamp1 = Clock::get()?.unix_timestamp;
        let loan_price = loan_feed
            .get_price_no_older_than(current_timestamp1, 60)
            .ok_or(ErrorCode::PythOffline)?;
        let loan_max_price = loan_price
            .price
            .checked_add(loan_price.conf as i64)
            .ok_or(ErrorCode::Overflow)?;
        let mut loan_max_value = loan_max_price
            .checked_mul(loan_qty)
            .ok_or(ErrorCode::Overflow)?;

        // Note : f64 should not be used in smart contracts, but we use it here so it gets displayed nicely in the logs.
        // lets get the maximum loan value based on computation
        // i.e {} * 10^({})
        // loan_max_value * 10^(loan_price.expo)
        let exponent: i32 = loan_price.expo;
        let result = (BASE as f64).powi(exponent.abs());
        let result = if exponent < 0 { 1.0 / result } else { result };
        let result_loan_value = loan_max_value as f64 * result;

        msg!(
            "The maximum loan value is {} * 10^({}) = {}.",
            loan_max_value,
            loan_price.expo,
            result_loan_value
        );

        // With high confidence, the minimum value of the collateral is
        // (price - conf) * collateral_qty * 10 ^ (expo).
        // Here is more explanation on confidence interval in Pyth:
        // https://docs.pyth.network/consume-data/best-practices
        let current_timestamp2 = Clock::get()?.unix_timestamp;
        let collateral_price = collateral_feed
            .get_price_no_older_than(current_timestamp2, 60)
            .ok_or(ErrorCode::PythOffline)?;
        let collateral_min_price = collateral_price
            .price
            .checked_sub(collateral_price.conf as i64)
            .ok_or(ErrorCode::Overflow)?;
        let mut collateral_min_value = collateral_min_price
            .checked_mul(collateral_qty)
            .ok_or(ErrorCode::Overflow)?;

        // Note : f64 should not be used in smart contracts, but we use it here so it gets displayed nicely in the logs.
        // lets get the minimum collateral value based on computation
        // i.e {} * 10^({})
        // i.e collateral_min_value * 10^(collateral_price.expo)
        let exponent: i32 = collateral_price.expo;
        let result = (BASE).powi(exponent.abs());
        let result: f64 = if exponent < 0 { 1.0 / result } else { result };
        let result_collateral_value = collateral_min_value as f64 * result;

        msg!(
            "The minimum collateral value is {} * 10^({}) = {}.",
            collateral_min_value,
            collateral_price.expo,
            result_collateral_value
        );

        // If the loan and collateral prices use different exponent,
        // normalize the value.
        if loan_price.expo > collateral_price.expo {
            let normalize = (10 as i64)
                .checked_pow((loan_price.expo - collateral_price.expo) as u32)
                .ok_or(ErrorCode::Overflow)?;
            collateral_min_value = collateral_min_value
                .checked_mul(normalize)
                .ok_or(ErrorCode::Overflow)?;
        } else if loan_price.expo < collateral_price.expo {
            let normalize = (10 as i64)
                .checked_pow((collateral_price.expo - loan_price.expo) as u32)
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

#[derive(Accounts)]
pub struct InitRequest<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + AdminConfig::INIT_SPACE
    )]
    pub config: Account<'info, AdminConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct QueryRequest<'info> {
    pub config: Account<'info, AdminConfig>,
    #[account(
        address = config.loan_price_feed_id @ ErrorCode::InvalidArgument
    )]
    pub pyth_loan_account: Account<'info, PriceFeed>,
    #[account(
        address = config.collateral_price_feed_id @ ErrorCode::InvalidArgument
    )]
    pub pyth_collateral_account: Account<'info, PriceFeed>,
}
