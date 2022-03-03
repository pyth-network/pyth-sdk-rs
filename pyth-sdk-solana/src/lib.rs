//! A Rust library for consuming price feeds from the [pyth.network](https://pyth.network/) oracle on the Solana network.
//!
//! Please see the [crates.io page](https://crates.io/crates/pyth-client/) for documentation and example usage.

pub use self::error::PythError;
pub use self::price_conf::PriceConf;

mod entrypoint;
mod error;
mod price_conf;

pub mod instruction;
pub mod processor;
pub mod state;

pub use state::*;