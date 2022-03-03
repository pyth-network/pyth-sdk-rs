//! A Rust library for consuming price feeds from the [pyth.network](https://pyth.network/) oracle on the Solana network.
//!
//! Please see the [crates.io page](https://crates.io/crates/pyth-sdk-solana/) for documentation and example usage.

pub use self::error::PythError;

mod error;
pub mod state;

pub mod entrypoint;
pub mod instruction;
pub mod processor;

// ID is Pyth Oracle mainnet id. This is also used in local testing.
solana_program::declare_id!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");


use state::load_price_account;

pub use pyth_sdk::{
    Price,
    PriceConf,
    PriceStatus,
};

#[cfg(target_arch = "bpf")]
use solana_program::{
    clock::Clock,
    sysvar::Sysvar,
};

/// Maximum acceptable slot difference before price is considered to be stale.
pub const MAX_SLOT_DIFFERENCE: u64 = 25;

/// Loads Pyth Price from the raw byte value of a Solana account.
pub fn load_price(data: &[u8]) -> Result<Price, PythError> {
    let price_account = load_price_account(data)?;

    #[allow(unused_mut)]
    let mut status = price_account.agg.status;

    #[cfg(target_arch = "bpf")]
    if matches!(status, PriceStatus::Trading)
        && Clock::get().unwrap().slot - price_account.agg.pub_slot > MAX_SLOT_DIFFERENCE
    {
        status = PriceStatus::Unknown;
    }

    Ok(Price {
        price: price_account.agg.price,
        conf: price_account.agg.conf,
        status,
        max_num_publishers: price_account.num,
        num_publishers: price_account.num_qt,
        ema_price: price_account.twap.val,
        ema_conf: price_account.twac.val as u64,
        expo: price_account.expo,
        product_id: price_account.prod.val,
    })
}
