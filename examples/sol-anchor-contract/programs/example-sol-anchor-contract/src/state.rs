use std::str::FromStr;
use anchor_lang::prelude::*;
use pyth_sdk_solana::state::load_price_account;

use crate::ErrorCode;
pub use pyth_sdk::Price;

#[account]
pub struct AdminConfig {
    pub loan_price_feed_id:       Pubkey,
    pub collateral_price_feed_id: Pubkey,
}

#[derive(Clone)]
pub struct PythPrice {
    pub price: Price,
}

impl anchor_lang::AccountDeserialize for PythPrice {
    fn try_deserialize_unchecked(data: &mut &[u8]) -> Result<Self>{
        let account = load_price_account(data)
            .map_err(|_x| error!(ErrorCode::PythError))?;
        let price = account.get_price_no_older_than(&Clock::get()?, 60)
            .ok_or(error!(ErrorCode::PythOffline))?;
        return Ok(PythPrice {price: price});
    }
}

impl anchor_lang::AccountSerialize for PythPrice {
    fn try_serialize<W: std::io::Write>(&self, _writer: &mut W,) -> std::result::Result<(), Error> {
        Err(error!(ErrorCode::TryToSerializePriceAccount))
    }
}

impl anchor_lang::Owner for PythPrice {
    fn owner() -> Pubkey {
        // CHECK: this is the pyth oracle address on solana devnet
        let oracle_addr = "gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s";
        return  Pubkey::from_str(&oracle_addr).unwrap();
    }
}
