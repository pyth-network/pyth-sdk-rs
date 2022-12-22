use std::str::FromStr;
use pyth_sdk::PriceFeed;
use anchor_lang::prelude::*;
use pyth_sdk_solana::state::load_price_account;

use crate::ErrorCode;

#[account]
pub struct AdminConfig {
    pub loan_price_feed_id:       Pubkey,
    pub collateral_price_feed_id: Pubkey,
}

#[derive(Clone)]
pub struct PythPriceFeed {
    pub feed: PriceFeed,
}

#[automatically_derived]
impl anchor_lang::AccountDeserialize for PythPriceFeed {
    fn try_deserialize_unchecked(data: &mut &[u8]) -> Result<Self>{
        let account = load_price_account(data)
            .map_err(|_x| error!(ErrorCode::PythError))?;
        // CHECK: using a dummy key for constructing PriceFeed
        let zeros: [u8; 32] = [0; 32];
        let dummy_key = Pubkey::new(&zeros);
        let feed = account.to_price_feed(&dummy_key);
        return Ok(PythPriceFeed {feed: feed});
    }
}

#[automatically_derived]
impl anchor_lang::AccountSerialize for PythPriceFeed {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W,) -> std::result::Result<(), Error> {
        Err(error!(ErrorCode::TryToSerializePriceAccount))
    }
}

#[automatically_derived]
impl anchor_lang::Owner for PythPriceFeed {
    fn owner() -> Pubkey {
        // CHECK: this is the pyth oracle address on solana devnet
        let oracle_addr = "gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s";
        return  Pubkey::from_str(&oracle_addr).unwrap();
    }
}
