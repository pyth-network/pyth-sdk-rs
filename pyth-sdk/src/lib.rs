use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use hex::FromHexError;
use schemars::JsonSchema;
use std::fmt;

pub mod utils;

mod price;
pub use price::Price;

#[derive(
    Copy,
    Clone,
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
pub struct Identifier(
    #[serde(with = "hex")]
    #[schemars(with = "String")]
    [u8; 32],
);

impl Identifier {
    pub fn new(bytes: [u8; 32]) -> Identifier {
        Identifier(bytes)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex<T: AsRef<[u8]>>(s: T) -> Result<Identifier, FromHexError> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(Identifier::new(bytes))
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0x")?;
        f.write_str(&self.to_hex())
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0x")?;
        f.write_str(&self.to_hex())
    }
}


/// Consists of 32 bytes and it is currently based on largest Public Key size on various
/// blockchains.
pub type PriceIdentifier = Identifier;

/// Consists of 32 bytes and it is currently based on largest Public Key size on various
/// blockchains.
pub type ProductIdentifier = Identifier;

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
    /// Current price aggregation publish time
    pub publish_time:       UnixTimestamp,
    /// Price exponent.
    pub expo:               i32,
    /// Maximum number of allowed publishers that can contribute to a price.
    pub max_num_publishers: u32,
    /// Number of publishers that made up current aggregate.
    pub num_publishers:     u32,
    /// Product account key.
    pub product_id:         ProductIdentifier,
    /// The current aggregation price.
    #[serde(with = "utils::as_string")] // To ensure accuracy on conversion to json.
    #[schemars(with = "String")]
    price:                  i64,
    /// Confidence interval around the current aggregation price.
    #[serde(with = "utils::as_string")]
    #[schemars(with = "String")]
    conf:                   u64,
    /// Exponentially moving average price.
    #[serde(with = "utils::as_string")]
    #[schemars(with = "String")]
    ema_price:              i64,
    /// Exponentially moving average confidence interval.
    #[serde(with = "utils::as_string")]
    #[schemars(with = "String")]
    ema_conf:               u64,
    /// Price of previous aggregate with Trading status.
    #[serde(with = "utils::as_string")]
    #[schemars(with = "String")]
    prev_price:             i64,
    /// Confidence interval of previous aggregate with Trading status.
    #[serde(with = "utils::as_string")]
    #[schemars(with = "String")]
    prev_conf:              u64,
    /// Publish time of previous aggregate with Trading status.
    prev_publish_time:      UnixTimestamp,
}

impl PriceFeed {
    /// Constructs a new Price Feed
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: PriceIdentifier,
        status: PriceStatus,
        publish_time: UnixTimestamp,
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
        prev_publish_time: UnixTimestamp,
    ) -> PriceFeed {
        PriceFeed {
            id,
            status,
            publish_time,
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
            prev_publish_time,
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

    /// Get the "unchecked" previous price with Trading status,
    /// along with the timestamp at which it was generated.
    ///
    /// WARNING:
    /// We make no guarantees about the unchecked price and confidence returned by
    /// this function: it could differ significantly from the current price.
    /// We strongly encourage you to use `get_current_price` instead.
    pub fn get_prev_price_unchecked(&self) -> (Price, UnixTimestamp) {
        (
            Price {
                price: self.prev_price,
                conf:  self.prev_conf,
                expo:  self.expo,
            },
            self.prev_publish_time,
        )
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_ser_then_deser_default() {
        let price_feed = PriceFeed::default();
        let ser = serde_json::to_string(&price_feed).unwrap();
        let deser: PriceFeed = serde_json::from_str(&ser).unwrap();
        assert_eq!(price_feed, deser);
    }

    #[test]
    pub fn test_ser_large_number() {
        let price_feed = PriceFeed {
            ema_conf: 1_234_567_000_000_000_789,
            ..PriceFeed::default()
        };
        let price_feed_json = serde_json::to_value(price_feed).unwrap();
        assert_eq!(
            price_feed_json["ema_conf"].as_str(),
            Some("1234567000000000789")
        );
    }

    #[test]
    pub fn test_deser_large_number() {
        let mut price_feed_json = serde_json::to_value(PriceFeed::default()).unwrap();
        price_feed_json["price"] = serde_json::Value::String(String::from("1000000000000000123"));
        let p: PriceFeed = serde_json::from_value(price_feed_json).unwrap();
        assert_eq!(p.price, 1_000_000_000_000_000_123);
    }

    #[test]
    pub fn test_ser_id_length_32_bytes() {
        let mut price_feed = PriceFeed::default();
        price_feed.id.0[0] = 106; // 0x6a
        let price_feed_json = serde_json::to_value(price_feed).unwrap();
        let id_str = price_feed_json["id"].as_str().unwrap();
        assert_eq!(id_str.len(), 64);
        assert_eq!(
            id_str,
            "6a00000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    pub fn test_deser_invalid_id_length_fails() {
        let mut price_feed_json = serde_json::to_value(PriceFeed::default()).unwrap();
        price_feed_json["id"] = serde_json::Value::String(String::from("1234567890"));
        assert!(serde_json::from_value::<PriceFeed>(price_feed_json).is_err());
    }

    #[test]
    pub fn test_identifier_from_hex_ok() {
        let id = Identifier::from_hex(
            "0a3f000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        assert_eq!(id.to_bytes()[0], 10);
    }

    #[test]
    pub fn test_identifier_from_hex_invalid_err() {
        let try_parse_odd = Identifier::from_hex("010"); // odd length
        assert_eq!(try_parse_odd, Err(FromHexError::OddLength));

        let try_parse_invalid_len = Identifier::from_hex("0a"); // length should be 32 bytes, 64
        assert_eq!(
            try_parse_invalid_len,
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    pub fn test_identifier_debug_fmt() {
        let mut id = Identifier::default();
        id.0[0] = 10;

        let id_str = format!("{:?}", id);
        assert_eq!(
            id_str,
            "0x0a00000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    pub fn test_identifier_display_fmt() {
        let mut id = Identifier::default();
        id.0[0] = 10;

        let id_str = format!("{}", id);
        assert_eq!(
            id_str,
            "0x0a00000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
