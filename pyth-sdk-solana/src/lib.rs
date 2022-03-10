//! A Rust library for consuming price feeds from the [pyth.network](https://pyth.network/) oracle on the Solana network.
//!
//! Please see the [crates.io page](https://crates.io/crates/pyth-sdk-solana/) for documentation and example usage.

pub use self::error::PythError;

mod error;
pub mod state;

use state::load_price_account;

pub use pyth_sdk::{
    Price,
    PriceConf,
    PriceStatus,
    ProductIdentifier,
};

/// Maximum valid slot period before price is considered to be stale.
pub const VALID_SLOT_PERIOD: u64 = 25;

/// Loads Pyth Price from the raw byte value of a Solana account.
pub fn load_price(data: &[u8]) -> Result<Price, PythError> {
    let price_account = load_price_account(data)?;

    Ok(price_account.to_price())
}
