//! A Rust library for consuming price feeds from the [pyth.network](https://pyth.network/) oracle on the Solana network.
//!
//! Please see the [crates.io page](https://crates.io/crates/pyth-sdk-solana/) for documentation and example usage.

pub use self::error::PythError;

mod error;
pub mod state;

pub mod entrypoint;
pub mod instruction;
pub mod processor;

// This is used only in local testing.
solana_program::declare_id!("PythC11111111111111111111111111111111111111");

use state::load_price_account;

pub use pyth_sdk::{
    Price,
    PriceConf,
    PriceStatus,
};

/// Maximum acceptable slot difference before price is considered to be stale.
pub const MAX_SLOT_DIFFERENCE: u64 = 25;

/// Loads Pyth Price from the raw byte value of a Solana account.
pub fn load_price(data: &[u8]) -> Result<Price, PythError> {
    let price_account = load_price_account(data)?;

    Ok(price_account.to_price())
}
