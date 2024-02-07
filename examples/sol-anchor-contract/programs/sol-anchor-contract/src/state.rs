use anchor_lang::prelude::*;
use pyth_sdk_solana::state::load_price_account;
use pyth_sdk_solana::state::SolanaPriceAccount;
use std::ops::Deref;
use std::str::FromStr;

use crate::ErrorCode;

#[account]
#[derive(InitSpace)]
pub struct AdminConfig {
    pub loan_price_feed_id: Pubkey,
    pub collateral_price_feed_id: Pubkey,
}

#[derive(Clone)]
pub struct PriceFeed(pyth_sdk::PriceFeed);

impl anchor_lang::Owner for PriceFeed {
    fn owner() -> Pubkey {
        // Make sure the owner is the pyth oracle account on solana devnet
        let oracle_addr = "gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s";
        return Pubkey::from_str(&oracle_addr).unwrap();
    }
}

impl anchor_lang::AccountDeserialize for PriceFeed {
    fn try_deserialize_unchecked(data: &mut &[u8]) -> Result<Self> {
        let account: &SolanaPriceAccount =
            load_price_account(data).map_err(|_x| error!(ErrorCode::PythError))?;

        // Use a dummy key since the key field will be removed from the SDK
        let zeros: [u8; 32] = [0; 32];
        let dummy_key = Pubkey::new(&zeros);
        let feed = account.to_price_feed(&dummy_key);
        return Ok(PriceFeed(feed));
    }
}

impl anchor_lang::AccountSerialize for PriceFeed {
    fn try_serialize<W: std::io::Write>(&self, _writer: &mut W) -> std::result::Result<(), Error> {
        Err(error!(ErrorCode::TryToSerializePriceAccount))
    }
}

impl Deref for PriceFeed {
    type Target = pyth_sdk::PriceFeed;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
