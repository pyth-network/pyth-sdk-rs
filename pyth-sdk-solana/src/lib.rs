//! A Rust library for consuming price feeds from the [pyth.network](https://pyth.network/) oracle on the Solana network.
//!
//! Please see the [crates.io page](https://crates.io/crates/pyth-sdk-solana/) for documentation and example usage.

pub use self::error::PythError;

mod error;
pub mod state;

use solana_program::account_info::{
    Account,
    AccountInfo,
    IntoAccountInfo,
};
use solana_program::pubkey::Pubkey;

use state::{
    load_price_account,
    GenericPriceAccount,
    SolanaPriceAccount,
};

pub use pyth_sdk::{
    Price,
    PriceFeed,
    PriceIdentifier,
    ProductIdentifier,
};

/// Maximum valid slot period before price is considered to be stale.
pub const VALID_SLOT_PERIOD: u64 = 25;

/// Loads Pyth Feed Price from Price Account Info.
#[deprecated(note = "solana-specific, use SolanaPriceAccount::account_info_to_feed instead.")]
pub fn load_price_feed_from_account_info(
    price_account_info: &AccountInfo,
) -> Result<PriceFeed, PythError> {
    SolanaPriceAccount::account_info_to_feed(price_account_info)
}

/// Loads Pyth Price Feed from Account when using Solana Client.
///
/// It is a helper function which constructs Account Info when reading Account in clients.
#[deprecated(note = "solana-specific, use SolanaPriceAccount::account_to_feed instead.")]
pub fn load_price_feed_from_account(
    price_key: &Pubkey,
    price_account: &mut impl Account,
) -> Result<PriceFeed, PythError> {
    SolanaPriceAccount::account_to_feed(price_key, price_account)
}

impl<const N: usize, T: 'static> GenericPriceAccount<N, T>
where
    T: Default,
    T: Copy,
{
    pub fn account_info_to_feed(price_account_info: &AccountInfo) -> Result<PriceFeed, PythError> {
        load_price_account::<N, T>(
            *price_account_info
                .try_borrow_data()
                .map_err(|_| PythError::InvalidAccountData)?,
        )
        .map(|acc| acc.to_price_feed(price_account_info.key))
    }

    pub fn account_to_feed(
        price_key: &Pubkey,
        price_account: &mut impl Account,
    ) -> Result<PriceFeed, PythError> {
        let price_account_info = (price_key, price_account).into_account_info();
        Self::account_info_to_feed(&price_account_info)
    }
}
