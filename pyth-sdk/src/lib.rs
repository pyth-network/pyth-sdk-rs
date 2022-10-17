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
    PartialOrd,
    Ord,
    Hash,
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
        write!(f, "0x{}", self.to_hex())
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

impl AsRef<[u8]> for Identifier {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
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
pub type DurationInSeconds = u64;

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
    pub id:    PriceIdentifier,
    /// Price.
    price:     Price,
    /// Exponentially-weighted moving average (EMA) price.
    ema_price: Price,
}

impl PriceFeed {
    /// Constructs a new Price Feed
    #[allow(clippy::too_many_arguments)]
    pub fn new(id: PriceIdentifier, price: Price, ema_price: Price) -> PriceFeed {
        PriceFeed {
            id,
            price,
            ema_price,
        }
    }


    /// Get the "unchecked" price and confidence interval as fixed-point numbers of the form
    /// a * 10^e along with its publish time.
    ///
    /// Returns a `Price` struct containing the current price, confidence interval, and the exponent
    /// for both numbers, and publish time. This method returns the latest price which may be from
    /// arbitrarily far in the past, and the caller should probably check the timestamp before using
    /// it.
    ///
    /// Please consider using `get_price_no_older_than` when possible.
    pub fn get_price_unchecked(&self) -> Price {
        self.price
    }


    /// Get the "unchecked" exponentially-weighted moving average (EMA) price and a confidence
    /// interval on the result along with its publish time.
    ///
    /// Returns the latest EMA price value which may be from arbitrarily far in the past, and the
    /// caller should probably check the timestamp before using it.
    ///
    /// At the moment, the confidence interval returned by this method is computed in
    /// a somewhat questionable way, so we do not recommend using it for high-value applications.
    ///
    /// Please consider using `get_ema_price_no_older_than` when possible.
    pub fn get_ema_price_unchecked(&self) -> Price {
        self.ema_price
    }

    /// Get the price as long as it was updated within `age` seconds of the
    /// `current_time`.
    ///
    /// This function is a sanity-checked version of `get_price_unchecked` which is
    /// useful in applications that require a sufficiently-recent price. Returns `None` if the
    /// price wasn't updated sufficiently recently.
    ///
    /// Returns a struct containing the latest available price, confidence interval and the exponent
    /// for both numbers, or `None` if no price update occurred within `age` seconds of the
    /// `current_time`.
    pub fn get_price_no_older_than(
        &self,
        current_time: UnixTimestamp,
        age: DurationInSeconds,
    ) -> Option<Price> {
        let price = self.get_price_unchecked();

        let time_diff_abs = (price.publish_time - current_time).abs() as u64;

        if time_diff_abs > age {
            return None;
        }

        Some(price)
    }

    /// Get the exponentially-weighted moving average (EMA) price as long as it was updated within
    /// `age` seconds of the `current_time`.
    ///
    /// This function is a sanity-checked version of `get_ema_price_unchecked` which is useful in
    /// applications that require a sufficiently-recent price. Returns `None` if the price
    /// wasn't updated sufficiently recently.
    ///
    /// Returns a struct containing the EMA price, confidence interval and the exponent
    /// for both numbers, or `None` if no price update occurred within `age` seconds of the
    /// `current_time`.
    pub fn get_ema_price_no_older_than(
        &self,
        current_time: UnixTimestamp,
        age: DurationInSeconds,
    ) -> Option<Price> {
        let price = self.get_ema_price_unchecked();

        let time_diff_abs = (price.publish_time - current_time).abs() as u64;

        if time_diff_abs > age {
            return None;
        }

        Some(price)
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
            ema_price: Price {
                conf: 1_234_567_000_000_000_789,
                ..Price::default()
            },
            ..PriceFeed::default()
        };
        let price_feed_json = serde_json::to_value(price_feed).unwrap();
        assert_eq!(
            price_feed_json["ema_price"]["conf"].as_str(),
            Some("1234567000000000789")
        );
    }

    #[test]
    pub fn test_deser_large_number() {
        let mut price_feed_json = serde_json::to_value(PriceFeed::default()).unwrap();
        price_feed_json["price"]["price"] =
            serde_json::Value::String(String::from("1000000000000000123"));
        let p: PriceFeed = serde_json::from_value(price_feed_json).unwrap();
        assert_eq!(p.get_price_unchecked().price, 1_000_000_000_000_000_123);
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
