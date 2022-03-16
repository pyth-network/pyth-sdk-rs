use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use schemars::JsonSchema;

mod price;
pub use price::Price;

/// Unique identifier for a price.
pub type PriceIdentifier = [u8; 32];

/// Consists of 32 bytes and it is currently based on largest Public Key size on various
/// blockchains.
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
pub struct PriceFeed {
    /// Unique identifier for this price.
    pub id:                 PriceIdentifier,
    /// Status of price (Trading is valid).
    pub status:             PriceStatus,
    /// Price exponent.
    pub expo:               i32,
    /// Maximum number of allowed publishers that can contribute to a price.
    pub max_num_publishers: u32,
    /// Number of publishers that made up current aggregate.
    pub num_publishers:     u32,
    /// Product account key.
    pub product_id:         ProductIdentifier,
    /// The current price.
    price:                  i64,
    /// Confidence interval around the price.
    conf:                   u64,
    /// Exponentially moving average price.
    ema_price:              i64,
    /// Exponentially moving average confidence interval.
    ema_conf:               u64,
}

impl PriceFeed {
    /// Constructs a new Price Feed
    pub fn new(
        id: PriceIdentifier,
        status: PriceStatus,
        expo: i32,
        max_num_publishers: u32,
        num_publishers: u32,
        product_id: ProductIdentifier,
        price: i64,
        conf: u64,
        ema_price: i64,
        ema_conf: u64,
    ) -> PriceFeed {
        PriceFeed {
            id,
            price,
            conf,
            status,
            expo,
            max_num_publishers,
            num_publishers,
            ema_price,
            ema_conf,
            product_id,
        }
    }

    /// Get the current price and confidence interval as fixed-point numbers of the form a *
    /// 10^e.
    ///
    /// Returns a struct containing the current price, confidence interval, and the exponent for
    /// both numbers. Returns `None` if price information is currently unavailable for any
    /// reason.
    pub fn get_current_price(&self) -> Option<Price> {
        if !matches!(self.status, PriceStatus::Trading) {
            None
        } else {
            Some(Price {
                price: self.price,
                conf:  self.conf,
                expo:  self.expo,
            })
        }
    }

    /// Get the "unchecked" current price and confidence interval as fixed-point numbers of the form
    /// a * 10^e.
    ///
    /// Returns a struct containing the current price, confidence interval, and the exponent for
    /// both numbers. This method returns the price value without checking availability of the
    /// price. This value might not be valid or updated when the price is not available.
    /// Please use `get_current_price` where possible.
    pub fn get_current_price_unchecked(&self) -> Price {
        Price {
            price: self.price,
            conf:  self.conf,
            expo:  self.expo,
        }
    }

    /// Get the exponential moving average price (ema_price) and a confidence interval on the
    /// result.
    ///
    /// Returns `None` if the ema price is currently unavailable.
    /// At the moment, the confidence interval returned by this method is computed in
    /// a somewhat questionable way, so we do not recommend using it for high-value applications.
    pub fn get_ema_price(&self) -> Option<Price> {
        // This method currently cannot return None, but may do so in the future.
        Some(Price {
            price: self.ema_price,
            conf:  self.ema_conf,
            expo:  self.expo,
        })
    }

    /// Get the "unchecked" exponential moving average price (ema_price) and a confidence interval
    /// on the result.
    ///
    /// Returns the price value without checking availability of the price.
    /// This value might not be valid or updated when the price is not available.
    /// Please use `get_ema_price` where possible.
    ///
    /// At the moment, the confidence interval returned by this method is computed in
    /// a somewhat questionable way, so we do not recommend using it for high-value applications.
    pub fn get_ema_price_unchecked(&self) -> Price {
        // This method currently cannot return None, but may do so in the future.
        Price {
            price: self.ema_price,
            conf:  self.ema_conf,
            expo:  self.expo,
        }
    }
}
