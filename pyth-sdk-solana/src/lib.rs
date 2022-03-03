//! A Rust library for consuming price feeds from the [pyth.network](https://pyth.network/) oracle on the Solana network.
//!
//! Please see the [crates.io page](https://crates.io/crates/pyth-client/) for documentation and example usage.

pub use self::price_conf::PriceConf;
pub use self::error::PythError;

mod entrypoint;
mod error;
mod price_conf;

pub mod processor;
pub mod instruction;

use std::mem::size_of;
use borsh::{BorshSerialize, BorshDeserialize};
use bytemuck::{
  cast_slice, from_bytes, try_cast_slice,
  Pod, PodCastError, Zeroable,
};

#[cfg(target_arch = "bpf")]
use solana_program::{clock::Clock, sysvar::Sysvar};

solana_program::declare_id!("PythC11111111111111111111111111111111111111");

pub const MAGIC               : u32   = 0xa1b2c3d4;
pub const VERSION_2           : u32   = 2;
pub const VERSION             : u32   = VERSION_2;
pub const MAP_TABLE_SIZE      : usize = 640;
pub const PROD_ACCT_SIZE      : usize = 512;
pub const PROD_HDR_SIZE       : usize = 48;
pub const PROD_ATTR_SIZE      : usize = PROD_ACCT_SIZE - PROD_HDR_SIZE;
pub const MAX_SLOT_DIFFERENCE : u64   = 25; 

/// The type of Pyth account determines what data it contains
#[derive(Copy, Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub enum AccountType
{
  Unknown,
  Mapping,
  Product,
  Price
}

impl Default for AccountType {
  fn default() -> Self {
    AccountType::Unknown
  }
}

/// The current status of a price feed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub enum PriceStatus
{
  /// The price feed is not currently updating for an unknown reason.
  Unknown,
  /// The price feed is updating as expected.
  Trading,
  /// The price feed is not currently updating because trading in the product has been halted.
  Halted,
  /// The price feed is not currently updating because an auction is setting the price.
  Auction
}

impl Default for PriceStatus {
  fn default() -> Self {
      PriceStatus::Unknown
  }
}

/// Status of any ongoing corporate actions.
/// (still undergoing dev)
#[derive(Copy, Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub enum CorpAction
{
  NoCorpAct
}

impl Default for CorpAction {
  fn default() -> Self {
      CorpAction::NoCorpAct
  }
}

/// The type of prices associated with a product -- each product may have multiple price feeds of different types.
#[derive(Copy, Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub enum PriceType
{
  Unknown,
  Price
}

impl Default for PriceType {
  fn default() -> Self {
      PriceType::Unknown
  }
}

/// Public key of a Solana account
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct AccKey
{
  pub val: [u8;32]
}

/// Mapping accounts form a linked-list containing the listing of all products on Pyth.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Mapping
{
  /// pyth magic number
  pub magic      : u32,
  /// program version
  pub ver        : u32,
  /// account type
  pub atype      : u32,
  /// account used size
  pub size       : u32,
  /// number of product accounts
  pub num        : u32,
  pub unused     : u32,
  /// next mapping account (if any)
  pub next       : AccKey,
  pub products   : [AccKey;MAP_TABLE_SIZE]
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for Mapping {}

#[cfg(target_endian = "little")]
unsafe impl Pod for Mapping {}


/// Product accounts contain metadata for a single product, such as its symbol ("Crypto.BTC/USD")
/// and its base/quote currencies.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Product
{
  /// pyth magic number
  pub magic      : u32,
  /// program version
  pub ver        : u32,
  /// account type
  pub atype      : u32,
  /// price account size
  pub size       : u32,
  /// first price account in list
  pub px_acc     : AccKey,
  /// key/value pairs of reference attr.
  pub attr       : [u8;PROD_ATTR_SIZE]
}

impl Product {
    pub fn iter(&self) -> AttributeIter {
        AttributeIter { attrs: &self.attr }
    }
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for Product {}

#[cfg(target_endian = "little")]
unsafe impl Pod for Product {}

/// A price and confidence at a specific slot. This struct can represent either a
/// publisher's contribution or the outcome of price aggregation.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct PriceInfo
{
  /// the current price. 
  /// For the aggregate price use price.get_current_price() whenever possible. It has more checks to make sure price is valid.
  pub price      : i64,
  /// confidence interval around the price.
  /// For the aggregate confidence use price.get_current_price() whenever possible. It has more checks to make sure price is valid.
  pub conf       : u64,
  /// status of price (Trading is valid).
  /// For the aggregate status use price.get_current_status() whenever possible.
  /// Price data can sometimes go stale and the function handles the status in such cases.
  pub status     : PriceStatus,
  /// notification of any corporate action
  pub corp_act   : CorpAction,
  pub pub_slot   : u64
}

/// The price and confidence contributed by a specific publisher.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct PriceComp
{
  /// key of contributing publisher
  pub publisher  : AccKey,
  /// the price used to compute the current aggregate price
  pub agg        : PriceInfo,
  /// The publisher's latest price. This price will be incorporated into the aggregate price
  /// when price aggregation runs next.
  pub latest     : PriceInfo

}

/// An exponentially-weighted moving average.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct Ema
{
  /// The current value of the EMA
  pub val        : i64,
  /// numerator state for next update
  pub numer          : i64,
  /// denominator state for next update
  pub denom          : i64
}

/// Price accounts represent a continuously-updating price feed for a product.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Price
{
  /// pyth magic number
  pub magic      : u32,
  /// program version
  pub ver        : u32,
  /// account type
  pub atype      : u32,
  /// price account size
  pub size       : u32,
  /// price or calculation type
  pub ptype      : PriceType,
  /// price exponent
  pub expo       : i32,
  /// number of component prices
  pub num        : u32,
  /// number of quoters that make up aggregate
  pub num_qt     : u32,
  /// slot of last valid (not unknown) aggregate price
  pub last_slot  : u64,
  /// valid slot-time of agg. price
  pub valid_slot : u64,
  /// time-weighted average price
  pub twap       : Ema,
  /// time-weighted average confidence interval
  pub twac       : Ema,
  /// space for future derived values
  pub drv1       : i64,
  /// space for future derived values
  pub drv2       : i64,
  /// product account key
  pub prod       : AccKey,
  /// next Price account in linked list
  pub next       : AccKey,
  /// valid slot of previous update
  pub prev_slot  : u64,
  /// aggregate price of previous update
  pub prev_price : i64,
  /// confidence interval of previous update
  pub prev_conf  : u64,
  /// space for future derived values
  pub drv3       : i64,
  /// aggregate price info
  pub agg        : PriceInfo,
  /// price components one per quoter
  pub comp       : [PriceComp;32]
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for Price {}

#[cfg(target_endian = "little")]
unsafe impl Pod for Price {}

impl Price {
  /**
   * Get the current status of the aggregate price.
   * If this lib is used on-chain it will mark price status as unknown if price has not been updated for a while.
   */
  pub fn get_current_price_status(&self) -> PriceStatus {
    #[cfg(target_arch = "bpf")]
    if matches!(self.agg.status, PriceStatus::Trading) &&
      Clock::get().unwrap().slot - self.agg.pub_slot > MAX_SLOT_DIFFERENCE {
      return PriceStatus::Unknown;
    }
    self.agg.status
  }

  /**
   * Get the current price and confidence interval as fixed-point numbers of the form a * 10^e.
   * Returns a struct containing the current price, confidence interval, and the exponent for both
   * numbers. Returns `None` if price information is currently unavailable for any reason.
   */
  pub fn get_current_price(&self) -> Option<PriceConf> {
    if !matches!(self.get_current_price_status(), PriceStatus::Trading) {
      None
    } else {
      Some(PriceConf {
        price: self.agg.price,
        conf: self.agg.conf,
        expo: self.expo
      })
    }
  }

  /**
   * Get the time-weighted average price (TWAP) and a confidence interval on the result.
   * Returns `None` if the twap is currently unavailable.
   *
   * At the moment, the confidence interval returned by this method is computed in
   * a somewhat questionable way, so we do not recommend using it for high-value applications.
   */
  pub fn get_twap(&self) -> Option<PriceConf> {
    // This method currently cannot return None, but may do so in the future.
    // Note that the twac is a positive number in i64, so safe to cast to u64.
    Some(PriceConf { price: self.twap.val, conf: self.twac.val as u64, expo: self.expo })
  }

  /**
   * Get the current price of this account in a different quote currency. If this account
   * represents the price of the product X/Z, and `quote` represents the price of the product Y/Z,
   * this method returns the price of X/Y. Use this method to get the price of e.g., mSOL/SOL from
   * the mSOL/USD and SOL/USD accounts.
   *
   * `result_expo` determines the exponent of the result, i.e., the number of digits below the decimal
   * point. This method returns `None` if either the price or confidence are too large to be
   * represented with the requested exponent.
   */
  pub fn get_price_in_quote(&self, quote: &Price, result_expo: i32) -> Option<PriceConf> {
    return match (self.get_current_price(), quote.get_current_price()) {
      (Some(base_price_conf), Some(quote_price_conf)) =>
        base_price_conf.div(&quote_price_conf)?.scale_to_exponent(result_expo),
      (_, _) => None,
    }
  }

  /**
   * Get the price of a basket of currencies. Each entry in `amounts` is of the form
   * `(price, qty, qty_expo)`, and the result is the sum of `price * qty * 10^qty_expo`.
   * The result is returned with exponent `result_expo`.
   *
   * An example use case for this function is to get the value of an LP token.
   */
  pub fn price_basket(amounts: &[(Price, i64, i32)], result_expo: i32) -> Option<PriceConf> {
    assert!(amounts.len() > 0);
    let mut res = PriceConf { price: 0, conf: 0, expo: result_expo };
    for i in 0..amounts.len() {
      res = res.add(
        &amounts[i].0.get_current_price()?.cmul(amounts[i].1, amounts[i].2)?.scale_to_exponent(result_expo)?
      )?
    }
    Some(res)
  }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct AccKeyU64
{
  pub val: [u64;4]
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for AccKeyU64 {}

#[cfg(target_endian = "little")]
unsafe impl Pod for AccKeyU64 {}

impl AccKey
{
  pub fn is_valid( &self ) -> bool  {
    match load::<AccKeyU64>( &self.val ) {
      Ok(k8) => k8.val[0]!=0 || k8.val[1]!=0 || k8.val[2]!=0 || k8.val[3]!=0,
      Err(_) => false,
    }
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

/** Get a `Mapping` account from the raw byte value of a Solana account. */
pub fn load_mapping(data: &[u8]) -> Result<&Mapping, PythError> {
  let pyth_mapping = load::<Mapping>(&data).map_err(|_| PythError::InvalidAccountData)?;

  if pyth_mapping.magic != MAGIC {
    return Err(PythError::InvalidAccountData);
  }
  if pyth_mapping.ver != VERSION_2 {
    return Err(PythError::BadVersionNumber);
  }
  if pyth_mapping.atype != AccountType::Mapping as u32 {
    return Err(PythError::WrongAccountType);
  }

  return Ok(pyth_mapping);
}

/** Get a `Product` account from the raw byte value of a Solana account. */
pub fn load_product(data: &[u8]) -> Result<&Product, PythError> {
  let pyth_product = load::<Product>(&data).map_err(|_| PythError::InvalidAccountData)?;

  if pyth_product.magic != MAGIC {
    return Err(PythError::InvalidAccountData);
  }
  if pyth_product.ver != VERSION_2 {
    return Err(PythError::BadVersionNumber);
  }
  if pyth_product.atype != AccountType::Product as u32 {
    return Err(PythError::WrongAccountType);
  }

  return Ok(pyth_product);
}

/** Get a `Price` account from the raw byte value of a Solana account. */
pub fn load_price(data: &[u8]) -> Result<&Price, PythError> {
  let pyth_price = load::<Price>(&data).map_err(|_| PythError::InvalidAccountData)?;

  if pyth_price.magic != MAGIC {
    return Err(PythError::InvalidAccountData);
  }
  if pyth_price.ver != VERSION_2 {
    return Err(PythError::BadVersionNumber);
  }
  if pyth_price.atype != AccountType::Price as u32 {
    return Err(PythError::WrongAccountType);
  }

  return Ok(pyth_price);
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
        return Some((key, val));
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
