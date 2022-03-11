use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use schemars::JsonSchema;

mod price_conf;
pub use price_conf::PriceConf;

/// Consists of 32 bytes and it is currently based on largest Public Key size on various blockchains.
pub type ProductIdentifier = [u8; 32];

/// Represents availability status of a price feed.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
    JsonSchema,
)]
#[repr(C)]
pub enum PriceStatus {
    /// The price feed is not currently updating for an unknown reason.
    Unknown,
    /// The price feed is updating as expected.
    Trading,
    /// The price feed is not currently updating because trading in the product has been halted.
    Halted,
    /// The price feed is not currently updating because an auction is setting the price.
    Auction,
}

impl Default for PriceStatus {
    fn default() -> Self {
        PriceStatus::Unknown
    }
}

/// Represents a current aggregation price from pyth publisher feeds.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
    JsonSchema,
)]
#[repr(C)]
pub struct Price {
    /// The current price.
    pub price:              i64,
    /// Confidence interval around the price.
    pub conf:               u64,
    /// Status of price (Trading is valid).
    pub status:             PriceStatus,
    /// Price exponent.
    pub expo:               i32,
    /// Maximum number of allowed publishers that can contribute to a price.
    pub max_num_publishers: u32,
    /// Number of publishers that made up current aggregate.
    pub num_publishers:     u32,
    /// Exponentially moving average price.
    pub ema_price:          i64,
    /// Exponentially moving average confidence interval.
    pub ema_conf:           u64,
    /// Product account key.
    pub product_id:         ProductIdentifier,
}

impl Price {
    /// Get the current price and confidence interval as fixed-point numbers of the form a *
    /// 10^e.
    /// 
    /// Returns a struct containing the current price, confidence interval, and the exponent for
    /// both numbers. Returns `None` if price information is currently unavailable for any
    /// reason.
    pub fn get_current_price(&self) -> Option<PriceConf> {
        if !matches!(self.status, PriceStatus::Trading) {
            None
        } else {
            Some(PriceConf {
                price: self.price,
                conf:  self.conf,
                expo:  self.expo,
            })
        }
    }

    /// Get the exponential moving average price (ema_price) and a confidence interval on the
    /// result.
    /// 
    /// Returns `None` if the ema price is currently unavailable.
    /// At the moment, the confidence interval returned by this method is computed in
    /// a somewhat questionable way, so we do not recommend using it for high-value applications.
    pub fn get_ema_price(&self) -> Option<PriceConf> {
        // This method currently cannot return None, but may do so in the future.
        Some(PriceConf {
            price: self.ema_price,
            conf:  self.ema_conf,
            expo:  self.expo,
        })
    }

    /// Get the current price of this account in a different quote currency.
    /// 
    /// If this account represents the price of the product X/Z, and `quote` represents the price
    /// of the product Y/Z, this method returns the price of X/Y. Use this method to get the
    /// price of e.g., mSOL/SOL from the mSOL/USD and SOL/USD accounts.
    /// 
    /// `result_expo` determines the exponent of the result, i.e., the number of digits below the
    /// decimal point. This method returns `None` if either the price or confidence are too
    /// large to be represented with the requested exponent.
    /// 
    /// Example:
    /// ```ignore
    /// let btc_usd: Price = ...;
    /// let eth_usd: Price = ...;
    /// // -8 is the desired exponent for the result
    /// let btc_eth: PriceConf = btc_usd.get_price_in_quote(&eth_usd, -8);
    /// println!("BTC/ETH price: ({} +- {}) x 10^{}", price.price, price.conf, price.expo);
    /// ```
    pub fn get_price_in_quote(&self, quote: &Price, result_expo: i32) -> Option<PriceConf> {
        match (self.get_current_price(), quote.get_current_price()) {
            (Some(base_price_conf), Some(quote_price_conf)) => base_price_conf
                .div(&quote_price_conf)?
                .scale_to_exponent(result_expo),
            (_, _) => None,
        }
    }

    /// Get the price of a basket of currencies.
    ///
    /// Each entry in `amounts` is of the form `(price, qty, qty_expo)`, and the result is the sum 
    /// of `price * qty * 10^qty_expo`. The result is returned with exponent `result_expo`.
    ///
    /// An example use case for this function is to get the value of an LP token.
    ///
    /// Example:
    /// ```ignore
    /// let btc_usd: Price = ...;
    /// let eth_usd: Price = ...;
    /// // Quantity of each asset in fixed-point a * 10^e.
    /// // This represents 0.1 BTC and .05 ETH.
    /// // -8 is desired exponent for result
    /// let basket_price: PriceConf = Price::price_basket(&[
    ///     (btc_usd, 10, -2),
    ///     (eth_usd, 5, -2)
    ///   ], -8);
    /// println!("0.1 BTC and 0.05 ETH are worth: ({} +- {}) x 10^{} USD",
    ///          basket_price.price, basket_price.conf, basket_price.expo);
    /// ```
    pub fn price_basket(amounts: &[(Price, i64, i32)], result_expo: i32) -> Option<PriceConf> {
        assert!(amounts.len() > 0);
        let mut res = PriceConf {
            price: 0,
            conf:  0,
            expo:  result_expo,
        };
        for i in 0..amounts.len() {
            res = res.add(
                &amounts[i]
                    .0
                    .get_current_price()?
                    .cmul(amounts[i].1, amounts[i].2)?
                    .scale_to_exponent(result_expo)?,
            )?
        }
        Some(res)
    }
}
