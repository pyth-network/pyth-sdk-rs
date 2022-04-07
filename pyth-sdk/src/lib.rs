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

/// Unix Timestamp is represented as number of seconds passed since Unix epoch (00:00:00 UTC on 1
/// Jan 1970). It is a signed integer because it's the standard in Unix systems and allows easier
/// time difference.
pub type UnixTimestamp = i64;

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
    /// Unix timestamp of current price aggregation time
    pub timestamp:          UnixTimestamp,
    /// Price exponent.
    pub expo:               i32,
    /// Maximum number of allowed publishers that can contribute to a price.
    pub max_num_publishers: u32,
    /// Number of publishers that made up current aggregate.
    pub num_publishers:     u32,
    /// Product account key.
    pub product_id:         ProductIdentifier,
    /// The current aggregation price.
    price:                  i64,
    /// Confidence interval around the current aggregation price.
    conf:                   u64,
    /// Exponentially moving average price.
    ema_price:              i64,
    /// Exponentially moving average confidence interval.
    ema_conf:               u64,
    /// Price of previous aggregate with Trading status.
    prev_price:             i64,
    /// Confidence interval of previous aggregate with Trading status.
    prev_conf:              u64,
    /// Unix timestamp of previous aggregate with Trading status.
    prev_timestamp:         UnixTimestamp,
}

impl PriceFeed {
    /// Constructs a new Price Feed
    pub fn new(
        id: PriceIdentifier,
        status: PriceStatus,
        timestamp: UnixTimestamp,
        expo: i32,
        max_num_publishers: u32,
        num_publishers: u32,
        product_id: ProductIdentifier,
        price: i64,
        conf: u64,
        ema_price: i64,
        ema_conf: u64,
        prev_price: i64,
        prev_conf: u64,
        prev_timestamp: UnixTimestamp,
    ) -> PriceFeed {
        PriceFeed {
            id,
            status,
            timestamp,
            expo,
            max_num_publishers,
            num_publishers,
            product_id,
            price,
            conf,
            ema_price,
            ema_conf,
            prev_price,
            prev_conf,
            prev_timestamp,
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

    /// Get the "unchecked" previous aggregate price with Trading status.
    ///
    /// Returns the previous aggregate price with the timestamp it was generated. The price might
    /// be invalid or inaccurate at the current time; You need to check timestamp when using it.
    /// Please use `get_current_price` where possible.
    pub fn get_prev_price_unchecked(&self) -> (Price, UnixTimestamp) {
        (
            Price {
                price: self.prev_price,
                conf:  self.prev_conf,
                expo:  self.expo,
            },
            self.prev_timestamp,
        )
    }
}
