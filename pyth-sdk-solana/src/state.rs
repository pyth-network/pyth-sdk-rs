//! Structures and functions for interacting with Solana on-chain account data.

use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use bytemuck::{
    cast_slice,
    from_bytes,
    try_cast_slice,
    Pod,
    PodCastError,
    Zeroable,
};
use pyth_sdk::{
    PriceIdentifier,
    UnixTimestamp,
};
use solana_program::clock::Clock;
use solana_program::pubkey::Pubkey;
use std::mem::size_of;

pub use pyth_sdk::{
    Price,
    PriceFeed,
};

use crate::PythError;

pub const MAGIC: u32 = 0xa1b2c3d4;
pub const VERSION_2: u32 = 2;
pub const VERSION: u32 = VERSION_2;
pub const MAP_TABLE_SIZE: usize = 640;
pub const PROD_ACCT_SIZE: usize = 512;
pub const PROD_HDR_SIZE: usize = 48;
pub const PROD_ATTR_SIZE: usize = PROD_ACCT_SIZE - PROD_HDR_SIZE;

/// The type of Pyth account determines what data it contains
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
)]
#[repr(C)]
pub enum AccountType {
    Unknown,
    Mapping,
    Product,
    Price,
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Unknown
    }
}

/// Status of any ongoing corporate actions.
/// (still undergoing dev)
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
)]
#[repr(C)]
pub enum CorpAction {
    NoCorpAct,
}

impl Default for CorpAction {
    fn default() -> Self {
        CorpAction::NoCorpAct
    }
}

/// The type of prices associated with a product -- each product may have multiple price feeds of
/// different types.
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
)]
#[repr(C)]
pub enum PriceType {
    Unknown,
    Price,
}

impl Default for PriceType {
    fn default() -> Self {
        PriceType::Unknown
    }
}


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
    /// A price component can be ignored if the confidence interval is too wide
    Ignored,
}

impl Default for PriceStatus {
    fn default() -> Self {
        PriceStatus::Unknown
    }
}

/// Mapping accounts form a linked-list containing the listing of all products on Pyth.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct MappingAccount {
    /// pyth magic number
    pub magic:    u32,
    /// program version
    pub ver:      u32,
    /// account type
    pub atype:    u32,
    /// account used size
    pub size:     u32,
    /// number of product accounts
    pub num:      u32,
    pub unused:   u32,
    /// next mapping account (if any)
    pub next:     Pubkey,
    pub products: [Pubkey; MAP_TABLE_SIZE],
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for MappingAccount {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for MappingAccount {
}


/// Product accounts contain metadata for a single product, such as its symbol ("Crypto.BTC/USD")
/// and its base/quote currencies.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ProductAccount {
    /// pyth magic number
    pub magic:  u32,
    /// program version
    pub ver:    u32,
    /// account type
    pub atype:  u32,
    /// price account size
    pub size:   u32,
    /// first price account in list
    pub px_acc: Pubkey,
    /// key/value pairs of reference attr.
    pub attr:   [u8; PROD_ATTR_SIZE],
}

impl ProductAccount {
    pub fn iter(&self) -> AttributeIter {
        AttributeIter { attrs: &self.attr }
    }
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for ProductAccount {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for ProductAccount {
}

/// A price and confidence at a specific slot. This struct can represent either a
/// publisher's contribution or the outcome of price aggregation.
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
)]
#[repr(C)]
pub struct PriceInfo {
    /// the current price.
    /// For the aggregate price use `get_price_no_older_than()` whenever possible. Accessing fields
    /// directly might expose you to stale or invalid prices.
    pub price:    i64,
    /// confidence interval around the price.
    /// For the aggregate confidence use `get_price_no_older_than()` whenever possible. Accessing
    /// fields directly might expose you to stale or invalid prices.
    pub conf:     u64,
    /// status of price (Trading is valid)
    pub status:   PriceStatus,
    /// notification of any corporate action
    pub corp_act: CorpAction,
    pub pub_slot: u64,
}

/// The price and confidence contributed by a specific publisher.
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
)]
#[repr(C)]
pub struct PriceComp {
    /// key of contributing publisher
    pub publisher: Pubkey,
    /// the price used to compute the current aggregate price
    pub agg:       PriceInfo,
    /// The publisher's latest price. This price will be incorporated into the aggregate price
    /// when price aggregation runs next.
    pub latest:    PriceInfo,
}

#[deprecated = "Type is renamed to Rational, please use the new name."]
pub type Ema = Rational;

/// An number represented as both `value` and also in rational as `numer/denom`.
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
)]
#[repr(C)]
pub struct Rational {
    pub val:   i64,
    pub numer: i64,
    pub denom: i64,
}

/// Price accounts represent a continuously-updating price feed for a product.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct PriceAccount {
    /// pyth magic number
    pub magic:          u32,
    /// program version
    pub ver:            u32,
    /// account type
    pub atype:          u32,
    /// price account size
    pub size:           u32,
    /// price or calculation type
    pub ptype:          PriceType,
    /// price exponent
    pub expo:           i32,
    /// number of component prices
    pub num:            u32,
    /// number of quoters that make up aggregate
    pub num_qt:         u32,
    /// slot of last valid (not unknown) aggregate price
    pub last_slot:      u64,
    /// valid slot-time of agg. price
    pub valid_slot:     u64,
    /// exponentially moving average price
    pub ema_price:      Rational,
    /// exponentially moving average confidence interval
    pub ema_conf:       Rational,
    /// unix timestamp of aggregate price
    pub timestamp:      i64,
    /// min publishers for valid price
    pub min_pub:        u8,
    /// space for future derived values
    pub drv2:           u8,
    /// space for future derived values
    pub drv3:           u16,
    /// space for future derived values
    pub drv4:           u32,
    /// product account key
    pub prod:           Pubkey,
    /// next Price account in linked list
    pub next:           Pubkey,
    /// valid slot of previous update
    pub prev_slot:      u64,
    /// aggregate price of previous update with TRADING status
    pub prev_price:     i64,
    /// confidence interval of previous update with TRADING status
    pub prev_conf:      u64,
    /// unix timestamp of previous aggregate with TRADING status
    pub prev_timestamp: i64,
    /// aggregate price info
    pub agg:            PriceInfo,
    /// price components one per quoter
    pub comp:           [PriceComp; 32],
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for PriceAccount {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for PriceAccount {
}

impl PriceAccount {
    pub fn get_publish_time(&self) -> UnixTimestamp {
        match self.agg.status {
            PriceStatus::Trading => self.timestamp,
            _ => self.prev_timestamp,
        }
    }

    /// Get the last valid price as long as it was updated within `slot_threshold` slots of the
    /// current slot.
    pub fn get_price_no_older_than(&self, clock: &Clock, slot_threshold: u64) -> Option<Price> {
        if self.agg.status == PriceStatus::Trading
            && self.agg.pub_slot >= clock.slot - slot_threshold
        {
            return Some(Price {
                conf:         self.agg.conf,
                expo:         self.expo,
                price:        self.agg.price,
                publish_time: self.timestamp,
            });
        }

        if self.prev_slot >= clock.slot - slot_threshold {
            return Some(Price {
                conf:         self.prev_conf,
                expo:         self.expo,
                price:        self.prev_price,
                publish_time: self.prev_timestamp,
            });
        }

        None
    }

    pub fn to_price_feed(&self, price_key: &Pubkey) -> PriceFeed {
        let status = self.agg.status;

        let price = match status {
            PriceStatus::Trading => Price {
                conf:         self.agg.conf,
                expo:         self.expo,
                price:        self.agg.price,
                publish_time: self.get_publish_time(),
            },
            _ => Price {
                conf:         self.prev_conf,
                expo:         self.expo,
                price:        self.prev_price,
                publish_time: self.get_publish_time(),
            },
        };

        let ema_price = Price {
            conf:         self.ema_conf.val as u64,
            expo:         self.expo,
            price:        self.ema_price.val,
            publish_time: self.get_publish_time(),
        };

        PriceFeed::new(PriceIdentifier::new(price_key.to_bytes()), price, ema_price)
    }
}

fn load<T: Pod>(data: &[u8]) -> Result<&T, PodCastError> {
    let size = size_of::<T>();
    if data.len() >= size {
        Ok(from_bytes(cast_slice::<u8, u8>(try_cast_slice(
            &data[0..size],
        )?)))
    } else {
        Err(PodCastError::SizeMismatch)
    }
}

/// Get a `Mapping` account from the raw byte value of a Solana account.
pub fn load_mapping_account(data: &[u8]) -> Result<&MappingAccount, PythError> {
    let pyth_mapping = load::<MappingAccount>(data).map_err(|_| PythError::InvalidAccountData)?;

    if pyth_mapping.magic != MAGIC {
        return Err(PythError::InvalidAccountData);
    }
    if pyth_mapping.ver != VERSION_2 {
        return Err(PythError::BadVersionNumber);
    }
    if pyth_mapping.atype != AccountType::Mapping as u32 {
        return Err(PythError::WrongAccountType);
    }

    Ok(pyth_mapping)
}

/// Get a `Product` account from the raw byte value of a Solana account.
pub fn load_product_account(data: &[u8]) -> Result<&ProductAccount, PythError> {
    let pyth_product = load::<ProductAccount>(data).map_err(|_| PythError::InvalidAccountData)?;

    if pyth_product.magic != MAGIC {
        return Err(PythError::InvalidAccountData);
    }
    if pyth_product.ver != VERSION_2 {
        return Err(PythError::BadVersionNumber);
    }
    if pyth_product.atype != AccountType::Product as u32 {
        return Err(PythError::WrongAccountType);
    }

    Ok(pyth_product)
}

/// Get a `Price` account from the raw byte value of a Solana account.
pub fn load_price_account(data: &[u8]) -> Result<&PriceAccount, PythError> {
    let pyth_price = load::<PriceAccount>(data).map_err(|_| PythError::InvalidAccountData)?;

    if pyth_price.magic != MAGIC {
        return Err(PythError::InvalidAccountData);
    }
    if pyth_price.ver != VERSION_2 {
        return Err(PythError::BadVersionNumber);
    }
    if pyth_price.atype != AccountType::Price as u32 {
        return Err(PythError::WrongAccountType);
    }

    Ok(pyth_price)
}

pub struct AttributeIter<'a> {
    attrs: &'a [u8],
}

impl<'a> Iterator for AttributeIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.attrs.is_empty() {
            return None;
        }
        let (key, data) = get_attr_str(self.attrs);
        let (val, data) = get_attr_str(data);
        self.attrs = data;
        Some((key, val))
    }
}

fn get_attr_str(buf: &[u8]) -> (&str, &[u8]) {
    if buf.is_empty() {
        return ("", &[]);
    }
    let len = buf[0] as usize;
    let str = std::str::from_utf8(&buf[1..len + 1]).expect("attr should be ascii or utf-8");
    let remaining_buf = &buf[len + 1..];
    (str, remaining_buf)
}

#[cfg(test)]
mod test {
    use pyth_sdk::{
        Identifier,
        Price,
        PriceFeed,
    };
    use solana_program::clock::Clock;
    use solana_program::pubkey::Pubkey;

    use super::{
        PriceAccount,
        PriceInfo,
        PriceStatus,
        Rational,
    };


    #[test]
    fn test_trading_price_to_price_feed() {
        let price_account = PriceAccount {
            expo: 5,
            agg: PriceInfo {
                price: 10,
                conf: 20,
                status: PriceStatus::Trading,
                ..Default::default()
            },
            timestamp: 200,
            prev_timestamp: 100,
            ema_price: Rational {
                val: 40,
                ..Default::default()
            },
            ema_conf: Rational {
                val: 50,
                ..Default::default()
            },
            prev_price: 60,
            prev_conf: 70,
            ..Default::default()
        };

        let pubkey = Pubkey::new_from_array([3; 32]);
        let price_feed = price_account.to_price_feed(&pubkey);

        assert_eq!(
            price_feed,
            PriceFeed::new(
                Identifier::new(pubkey.to_bytes()),
                Price {
                    conf:         20,
                    price:        10,
                    expo:         5,
                    publish_time: 200,
                },
                Price {
                    conf:         50,
                    price:        40,
                    expo:         5,
                    publish_time: 200,
                }
            )
        );
    }

    #[test]
    fn test_non_trading_price_to_price_feed() {
        let price_account = PriceAccount {
            expo: 5,
            agg: PriceInfo {
                price: 10,
                conf: 20,
                status: PriceStatus::Unknown,
                ..Default::default()
            },
            timestamp: 200,
            prev_timestamp: 100,
            ema_price: Rational {
                val: 40,
                ..Default::default()
            },
            ema_conf: Rational {
                val: 50,
                ..Default::default()
            },
            prev_price: 60,
            prev_conf: 70,
            ..Default::default()
        };

        let pubkey = Pubkey::new_from_array([3; 32]);
        let price_feed = price_account.to_price_feed(&pubkey);

        assert_eq!(
            price_feed,
            PriceFeed::new(
                Identifier::new(pubkey.to_bytes()),
                Price {
                    conf:         70,
                    price:        60,
                    expo:         5,
                    publish_time: 100,
                },
                Price {
                    conf:         50,
                    price:        40,
                    expo:         5,
                    publish_time: 100,
                }
            )
        );
    }

    #[test]
    fn test_happy_use_latest_price_in_price_no_older_than() {
        let price_account = PriceAccount {
            expo: 5,
            agg: PriceInfo {
                price: 10,
                conf: 20,
                status: PriceStatus::Trading,
                pub_slot: 1,
                ..Default::default()
            },
            timestamp: 200,
            prev_timestamp: 100,
            prev_price: 60,
            prev_conf: 70,
            ..Default::default()
        };

        let clock = Clock {
            slot: 5,
            ..Default::default()
        };

        assert_eq!(
            price_account.get_price_no_older_than(&clock, 4),
            Some(Price {
                conf:         20,
                expo:         5,
                price:        10,
                publish_time: 200,
            })
        );
    }

    #[test]
    fn test_happy_use_prev_price_in_price_no_older_than() {
        let price_account = PriceAccount {
            expo: 5,
            agg: PriceInfo {
                price: 10,
                conf: 20,
                status: PriceStatus::Unknown,
                pub_slot: 3,
                ..Default::default()
            },
            timestamp: 200,
            prev_timestamp: 100,
            prev_price: 60,
            prev_conf: 70,
            prev_slot: 1,
            ..Default::default()
        };

        let clock = Clock {
            slot: 5,
            ..Default::default()
        };

        assert_eq!(
            price_account.get_price_no_older_than(&clock, 4),
            Some(Price {
                conf:         70,
                expo:         5,
                price:        60,
                publish_time: 100,
            })
        );
    }

    #[test]
    fn test_sad_cur_price_unknown_in_price_no_older_than() {
        let price_account = PriceAccount {
            expo: 5,
            agg: PriceInfo {
                price: 10,
                conf: 20,
                status: PriceStatus::Unknown,
                pub_slot: 3,
                ..Default::default()
            },
            timestamp: 200,
            prev_timestamp: 100,
            prev_price: 60,
            prev_conf: 70,
            prev_slot: 1,
            ..Default::default()
        };

        let clock = Clock {
            slot: 5,
            ..Default::default()
        };

        // current price is unknown, prev price is too stale
        assert_eq!(price_account.get_price_no_older_than(&clock, 3), None);
    }

    #[test]
    fn test_sad_cur_price_stale_in_price_no_older_than() {
        let price_account = PriceAccount {
            expo: 5,
            agg: PriceInfo {
                price: 10,
                conf: 20,
                status: PriceStatus::Trading,
                pub_slot: 3,
                ..Default::default()
            },
            timestamp: 200,
            prev_timestamp: 100,
            prev_price: 60,
            prev_conf: 70,
            prev_slot: 1,
            ..Default::default()
        };

        let clock = Clock {
            slot: 5,
            ..Default::default()
        };

        assert_eq!(price_account.get_price_no_older_than(&clock, 1), None);
    }
}
