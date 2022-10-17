use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use schemars::JsonSchema;

use crate::{
    utils,
    UnixTimestamp,
};

// Constants for working with pyth's number representation
const PD_EXPO: i32 = -9;
const PD_SCALE: u64 = 1_000_000_000;
const MAX_PD_V_U64: u64 = (1 << 28) - 1;

/// A price with a degree of uncertainty at a certain time, represented as a price +- a confidence
/// interval.
///
/// Please refer to the documentation at https://docs.pyth.network/consumers/best-practices for
/// using this price safely.
///
/// The confidence interval roughly corresponds to the standard error of a normal distribution.
/// Both the price and confidence are stored in a fixed-point numeric representation, `x *
/// 10^expo`, where `expo` is the exponent. For example:
///
/// ```
/// use pyth_sdk::Price;
/// Price { price: 12345, conf: 267, expo: -2, publish_time: 100 }; // represents 123.45 +- 2.67 published at UnixTimestamp 100
/// Price { price: 123, conf: 1, expo: 2,  publish_time: 100 }; // represents 12300 +- 100 published at UnixTimestamp 100
/// ```
///
/// `Price` supports a limited set of mathematical operations. All of these operations will
/// propagate any uncertainty in the arguments into the result. However, the uncertainty in the
/// result may overestimate the true uncertainty (by at most a factor of `sqrt(2)`) due to
/// computational limitations. Furthermore, all of these operations may return `None` if their
/// result cannot be represented within the numeric representation (e.g., the exponent is so
/// small that the price does not fit into an i64). Users of these methods should (1) select
/// their exponents to avoid this problem, and (2) handle the `None` case gracefully.
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    PartialEq,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
    JsonSchema,
)]
pub struct Price {
    /// Price.
    #[serde(with = "utils::as_string")] // To ensure accuracy on conversion to json.
    #[schemars(with = "String")]
    pub price:        i64,
    /// Confidence interval.
    #[serde(with = "utils::as_string")]
    #[schemars(with = "String")]
    pub conf:         u64,
    /// Exponent.
    pub expo:         i32,
    /// Publish time.
    pub publish_time: UnixTimestamp,
}

impl Price {
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
    /// let btc_eth: Price = btc_usd.get_price_in_quote(&eth_usd, -8);
    /// println!("BTC/ETH price: ({} +- {}) x 10^{}", price.price, price.conf, price.expo);
    /// ```
    pub fn get_price_in_quote(&self, quote: &Price, result_expo: i32) -> Option<Price> {
        self.div(quote)?.scale_to_exponent(result_expo)
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
    /// let basket_price: Price = Price::price_basket(&[
    ///     (btc_usd, 10, -2),
    ///     (eth_usd, 5, -2)
    ///   ], -8);
    /// println!("0.1 BTC and 0.05 ETH are worth: ({} +- {}) x 10^{} USD",
    ///          basket_price.price, basket_price.conf, basket_price.expo);
    /// ```
    pub fn price_basket(amounts: &[(Price, i64, i32)], result_expo: i32) -> Option<Price> {
        if amounts.is_empty() {
            return None;
        }

        let mut res = Price {
            price:        0,
            conf:         0,
            expo:         result_expo,
            publish_time: amounts[0].0.publish_time,
        };
        for amount in amounts {
            res = res.add(
                &amount
                    .0
                    .cmul(amount.1, amount.2)?
                    .scale_to_exponent(result_expo)?,
            )?
        }
        Some(res)
    }

    /// Divide this price by `other` while propagating the uncertainty in both prices into the
    /// result.
    ///
    /// This method will automatically select a reasonable exponent for the result. If both
    /// `self` and `other` are normalized, the exponent is `self.expo + PD_EXPO - other.expo`
    /// (i.e., the fraction has `PD_EXPO` digits of additional precision). If they are not
    /// normalized, this method will normalize them, resulting in an unpredictable result
    /// exponent. If the result is used in a context that requires a specific exponent,
    /// please call `scale_to_exponent` on it.
    pub fn div(&self, other: &Price) -> Option<Price> {
        // Price is not guaranteed to store its price/confidence in normalized form.
        // Normalize them here to bound the range of price/conf, which is required to perform
        // arithmetic operations.
        let base = self.normalize()?;
        let other = other.normalize()?;

        if other.price == 0 {
            return None;
        }

        // These use at most 27 bits each
        let (base_price, base_sign) = Price::to_unsigned(base.price);
        let (other_price, other_sign) = Price::to_unsigned(other.price);

        // Compute the midprice, base in terms of other.
        // Uses at most 57 bits
        let midprice = base_price.checked_mul(PD_SCALE)?.checked_div(other_price)?;
        let midprice_expo = base.expo.checked_sub(other.expo)?.checked_add(PD_EXPO)?;

        // Compute the confidence interval.
        // This code uses the 1-norm instead of the 2-norm for computational reasons.
        // Let p +- a and q +- b be the two arguments to this method. The correct
        // formula is p/q * sqrt( (a/p)^2 + (b/q)^2 ). This quantity
        // is difficult to compute due to the sqrt and overflow/underflow considerations.
        //
        // This code instead computes p/q * (a/p + b/q) = a/q + pb/q^2 .
        // This quantity is at most a factor of sqrt(2) greater than the correct result, which
        // shouldn't matter considering that confidence intervals are typically ~0.1% of the price.

        // This uses 57 bits and has an exponent of PD_EXPO.
        let other_confidence_pct: u64 =
            other.conf.checked_mul(PD_SCALE)?.checked_div(other_price)?;

        // first term is 57 bits, second term is 57 + 58 - 29 = 86 bits. Same exponent as the
        // midprice. Note: the computation of the 2nd term consumes about 3k ops. We may
        // want to optimize this.
        let conf = (base.conf.checked_mul(PD_SCALE)?.checked_div(other_price)? as u128)
            .checked_add(
                (other_confidence_pct as u128)
                    .checked_mul(midprice as u128)?
                    .checked_div(PD_SCALE as u128)?,
            )?;

        // Note that this check only fails if an argument's confidence interval was >> its price,
        // in which case None is a reasonable result, as we have essentially 0 information about the
        // price.
        if conf < (u64::MAX as u128) {
            Some(Price {
                price:        (midprice as i64)
                    .checked_mul(base_sign)?
                    .checked_mul(other_sign)?,
                conf:         conf as u64,
                expo:         midprice_expo,
                publish_time: self.publish_time.min(other.publish_time),
            })
        } else {
            None
        }
    }

    /// Add `other` to this, propagating uncertainty in both prices.
    ///
    /// Requires both `Price`s to have the same exponent -- use `scale_to_exponent` on
    /// the arguments if necessary.
    ///
    /// TODO: could generalize this method to support different exponents.
    pub fn add(&self, other: &Price) -> Option<Price> {
        assert_eq!(self.expo, other.expo);

        let price = self.price.checked_add(other.price)?;
        // The conf should technically be sqrt(a^2 + b^2), but that's harder to compute.
        let conf = self.conf.checked_add(other.conf)?;
        Some(Price {
            price,
            conf,
            expo: self.expo,
            publish_time: self.publish_time.min(other.publish_time),
        })
    }

    /// Multiply this `Price` by a constant `c * 10^e`.
    pub fn cmul(&self, c: i64, e: i32) -> Option<Price> {
        self.mul(&Price {
            price:        c,
            conf:         0,
            expo:         e,
            publish_time: self.publish_time,
        })
    }

    /// Multiply this `Price` by `other`, propagating any uncertainty.
    pub fn mul(&self, other: &Price) -> Option<Price> {
        // Price is not guaranteed to store its price/confidence in normalized form.
        // Normalize them here to bound the range of price/conf, which is required to perform
        // arithmetic operations.
        let base = self.normalize()?;
        let other = other.normalize()?;

        // These use at most 27 bits each
        let (base_price, base_sign) = Price::to_unsigned(base.price);
        let (other_price, other_sign) = Price::to_unsigned(other.price);

        // Uses at most 27*2 = 54 bits
        let midprice = base_price.checked_mul(other_price)?;
        let midprice_expo = base.expo.checked_add(other.expo)?;

        // Compute the confidence interval.
        // This code uses the 1-norm instead of the 2-norm for computational reasons.
        // Note that this simplifies: pq * (a/p + b/q) = qa + pb
        // 27*2 + 1 bits
        let conf = base
            .conf
            .checked_mul(other_price)?
            .checked_add(other.conf.checked_mul(base_price)?)?;

        Some(Price {
            price: (midprice as i64)
                .checked_mul(base_sign)?
                .checked_mul(other_sign)?,
            conf,
            expo: midprice_expo,
            publish_time: self.publish_time.min(other.publish_time),
        })
    }

    /// Get a copy of this struct where the price and confidence
    /// have been normalized to be between `MIN_PD_V_I64` and `MAX_PD_V_I64`.
    pub fn normalize(&self) -> Option<Price> {
        // signed division is very expensive in op count
        let (mut p, s) = Price::to_unsigned(self.price);
        let mut c = self.conf;
        let mut e = self.expo;

        while p > MAX_PD_V_U64 || c > MAX_PD_V_U64 {
            p = p.checked_div(10)?;
            c = c.checked_div(10)?;
            e = e.checked_add(1)?;
        }

        Some(Price {
            price:        (p as i64).checked_mul(s)?,
            conf:         c,
            expo:         e,
            publish_time: self.publish_time,
        })
    }

    /// Scale this price/confidence so that its exponent is `target_expo`.
    ///
    /// Return `None` if this number is outside the range of numbers representable in `target_expo`,
    /// which will happen if `target_expo` is too small.
    ///
    /// Warning: if `target_expo` is significantly larger than the current exponent, this
    /// function will return 0 +- 0.
    pub fn scale_to_exponent(&self, target_expo: i32) -> Option<Price> {
        let mut delta = target_expo.checked_sub(self.expo)?;
        if delta >= 0 {
            let mut p = self.price;
            let mut c = self.conf;
            // 2nd term is a short-circuit to bound op consumption
            while delta > 0 && (p != 0 || c != 0) {
                p = p.checked_div(10)?;
                c = c.checked_div(10)?;
                delta = delta.checked_sub(1)?;
            }

            Some(Price {
                price:        p,
                conf:         c,
                expo:         target_expo,
                publish_time: self.publish_time,
            })
        } else {
            let mut p = self.price;
            let mut c = self.conf;

            // Either p or c == None will short-circuit to bound op consumption
            while delta < 0 {
                p = p.checked_mul(10)?;
                c = c.checked_mul(10)?;
                delta = delta.checked_add(1)?;
            }

            Some(Price {
                price:        p,
                conf:         c,
                expo:         target_expo,
                publish_time: self.publish_time,
            })
        }
    }

    /// Helper function to convert signed integers to unsigned and a sign bit, which simplifies
    /// some of the computations above.
    fn to_unsigned(x: i64) -> (u64, i64) {
        if x == i64::MIN {
            // special case because i64::MIN == -i64::MIN
            (i64::MAX as u64 + 1, -1)
        } else if x < 0 {
            (-x as u64, -1)
        } else {
            (x as u64, 1)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::price::{
        Price,
        MAX_PD_V_U64,
        PD_EXPO,
        PD_SCALE,
    };

    const MAX_PD_V_I64: i64 = MAX_PD_V_U64 as i64;
    const MIN_PD_V_I64: i64 = -MAX_PD_V_I64;

    fn pc(price: i64, conf: u64, expo: i32) -> Price {
        Price {
            price,
            conf,
            expo,
            publish_time: 0,
        }
    }

    fn pc_scaled(price: i64, conf: u64, cur_expo: i32, expo: i32) -> Price {
        Price {
            price,
            conf,
            expo: cur_expo,
            publish_time: 0,
        }
        .scale_to_exponent(expo)
        .unwrap()
    }

    #[test]
    fn test_normalize() {
        fn succeeds(price1: Price, expected: Price) {
            assert_eq!(price1.normalize().unwrap(), expected);
        }

        fn fails(price1: Price) {
            assert_eq!(price1.normalize(), None);
        }

        succeeds(
            pc(2 * (PD_SCALE as i64), 3 * PD_SCALE, 0),
            pc(2 * (PD_SCALE as i64) / 100, 3 * PD_SCALE / 100, 2),
        );

        succeeds(
            pc(-2 * (PD_SCALE as i64), 3 * PD_SCALE, 0),
            pc(-2 * (PD_SCALE as i64) / 100, 3 * PD_SCALE / 100, 2),
        );

        // the i64 / u64 max values are a factor of 10^11 larger than MAX_PD_V
        let expo = -(PD_EXPO - 2);
        let scale_i64 = (PD_SCALE as i64) * 100;
        let scale_u64 = scale_i64 as u64;
        succeeds(pc(i64::MAX, 1, 0), pc(i64::MAX / scale_i64, 0, expo));
        succeeds(pc(i64::MIN, 1, 0), pc(i64::MIN / scale_i64, 0, expo));
        succeeds(pc(1, u64::MAX, 0), pc(0, u64::MAX / scale_u64, expo));

        // exponent overflows
        succeeds(
            pc(i64::MAX, 1, i32::MAX - expo),
            pc(i64::MAX / scale_i64, 0, i32::MAX),
        );
        fails(pc(i64::MAX, 1, i32::MAX - expo + 1));
        succeeds(
            pc(i64::MAX, 1, i32::MIN),
            pc(i64::MAX / scale_i64, 0, i32::MIN + expo),
        );

        succeeds(
            pc(1, u64::MAX, i32::MAX - expo),
            pc(0, u64::MAX / scale_u64, i32::MAX),
        );
        fails(pc(1, u64::MAX, i32::MAX - expo + 1));

        // Check timestamp won't change after normalize
        let p = Price {
            publish_time: 100,
            ..Default::default()
        };

        assert_eq!(p.normalize().unwrap().publish_time, 100);
    }

    #[test]
    fn test_scale_to_exponent() {
        fn succeeds(price1: Price, target: i32, expected: Price) {
            assert_eq!(price1.scale_to_exponent(target).unwrap(), expected);
        }

        fn fails(price1: Price, target: i32) {
            assert_eq!(price1.scale_to_exponent(target), None);
        }

        succeeds(pc(1234, 1234, 0), 0, pc(1234, 1234, 0));
        succeeds(pc(1234, 1234, 0), 1, pc(123, 123, 1));
        succeeds(pc(1234, 1234, 0), 2, pc(12, 12, 2));
        succeeds(pc(-1234, 1234, 0), 2, pc(-12, 12, 2));
        succeeds(pc(1234, 1234, 0), 4, pc(0, 0, 4));
        succeeds(pc(1234, 1234, 0), -1, pc(12340, 12340, -1));
        succeeds(pc(1234, 1234, 0), -2, pc(123400, 123400, -2));
        succeeds(pc(1234, 1234, 0), -8, pc(123400000000, 123400000000, -8));
        // insufficient precision to represent the result in this exponent
        fails(pc(1234, 1234, 0), -20);
        fails(pc(1234, 0, 0), -20);
        fails(pc(0, 1234, 0), -20);

        // fails because exponent delta overflows
        fails(pc(1, 1, i32::MIN), i32::MAX);

        // Check timestamp won't change after scale to exponent
        let p = Price {
            publish_time: 100,
            ..pc(1234, 1234, 0)
        };

        assert_eq!(p.scale_to_exponent(2).unwrap().publish_time, 100);
    }

    #[test]
    fn test_div() {
        fn succeeds(price1: Price, price2: Price, expected: Price) {
            assert_eq!(price1.div(&price2).unwrap(), expected);
        }

        fn fails(price1: Price, price2: Price) {
            let result = price1.div(&price2);
            assert_eq!(result, None);
        }

        succeeds(pc(1, 1, 0), pc(1, 1, 0), pc_scaled(1, 2, 0, PD_EXPO));
        succeeds(pc(1, 1, -8), pc(1, 1, -8), pc_scaled(1, 2, 0, PD_EXPO));
        succeeds(pc(10, 1, 0), pc(1, 1, 0), pc_scaled(10, 11, 0, PD_EXPO));
        succeeds(pc(1, 1, 1), pc(1, 1, 0), pc_scaled(10, 20, 0, PD_EXPO + 1));
        succeeds(pc(1, 1, 0), pc(5, 1, 0), pc_scaled(20, 24, -2, PD_EXPO));

        // Negative numbers
        succeeds(pc(-1, 1, 0), pc(1, 1, 0), pc_scaled(-1, 2, 0, PD_EXPO));
        succeeds(pc(1, 1, 0), pc(-1, 1, 0), pc_scaled(-1, 2, 0, PD_EXPO));
        succeeds(pc(-1, 1, 0), pc(-1, 1, 0), pc_scaled(1, 2, 0, PD_EXPO));

        // Different exponents in the two inputs
        succeeds(
            pc(100, 10, -8),
            pc(2, 1, -7),
            pc_scaled(500_000_000, 300_000_000, -8, PD_EXPO - 1),
        );
        succeeds(
            pc(100, 10, -4),
            pc(2, 1, 0),
            pc_scaled(500_000, 300_000, -8, PD_EXPO + -4),
        );

        // Test with end range of possible inputs where the output should not lose precision.
        succeeds(
            pc(MAX_PD_V_I64, MAX_PD_V_U64, 0),
            pc(MAX_PD_V_I64, MAX_PD_V_U64, 0),
            pc_scaled(1, 2, 0, PD_EXPO),
        );
        succeeds(
            pc(MAX_PD_V_I64, MAX_PD_V_U64, 0),
            pc(1, 1, 0),
            pc_scaled(MAX_PD_V_I64, 2 * MAX_PD_V_U64, 0, PD_EXPO),
        );
        succeeds(
            pc(1, 1, 0),
            pc(MAX_PD_V_I64, MAX_PD_V_U64, 0),
            pc(
                (PD_SCALE as i64) / MAX_PD_V_I64,
                2 * (PD_SCALE / MAX_PD_V_U64),
                PD_EXPO,
            ),
        );

        succeeds(
            pc(MIN_PD_V_I64, MAX_PD_V_U64, 0),
            pc(MIN_PD_V_I64, MAX_PD_V_U64, 0),
            pc_scaled(1, 2, 0, PD_EXPO),
        );
        succeeds(
            pc(MIN_PD_V_I64, MAX_PD_V_U64, 0),
            pc(1, 1, 0),
            pc_scaled(MIN_PD_V_I64, 2 * MAX_PD_V_U64, 0, PD_EXPO),
        );
        succeeds(
            pc(1, 1, 0),
            pc(MIN_PD_V_I64, MAX_PD_V_U64, 0),
            pc(
                (PD_SCALE as i64) / MIN_PD_V_I64,
                2 * (PD_SCALE / MAX_PD_V_U64),
                PD_EXPO,
            ),
        );

        succeeds(
            pc(1, MAX_PD_V_U64, 0),
            pc(1, MAX_PD_V_U64, 0),
            pc_scaled(1, 2 * MAX_PD_V_U64, 0, PD_EXPO),
        );
        // This fails because the confidence interval is too large to be represented in PD_EXPO
        fails(pc(MAX_PD_V_I64, MAX_PD_V_U64, 0), pc(1, MAX_PD_V_U64, 0));

        // Unnormalized tests below here

        // More realistic inputs (get BTC price in ETH)
        let ten_e7: i64 = 10000000;
        let uten_e7: u64 = 10000000;
        succeeds(
            pc(520010 * ten_e7, 310 * uten_e7, -8),
            pc(38591 * ten_e7, 18 * uten_e7, -8),
            pc(1347490347, 1431804, -8),
        );

        // Test with end range of possible inputs to identify overflow
        // These inputs will lose precision due to the initial normalization.
        // Get the rounded versions of these inputs in order to compute the expected results.
        let normed = pc(i64::MAX, u64::MAX, 0).normalize().unwrap();

        succeeds(
            pc(i64::MAX, u64::MAX, 0),
            pc(i64::MAX, u64::MAX, 0),
            pc_scaled(1, 4, 0, PD_EXPO),
        );
        succeeds(
            pc(i64::MAX, u64::MAX, 0),
            pc(1, 1, 0),
            pc_scaled(
                normed.price,
                3 * (normed.price as u64),
                normed.expo,
                normed.expo + PD_EXPO,
            ),
        );
        succeeds(
            pc(1, 1, 0),
            pc(i64::MAX, u64::MAX, 0),
            pc(
                (PD_SCALE as i64) / normed.price,
                3 * (PD_SCALE / (normed.price as u64)),
                PD_EXPO - normed.expo,
            ),
        );

        succeeds(
            pc(i64::MAX, 1, 0),
            pc(i64::MAX, 1, 0),
            pc_scaled(1, 0, 0, PD_EXPO),
        );
        succeeds(
            pc(i64::MAX, 1, 0),
            pc(1, 1, 0),
            pc_scaled(
                normed.price,
                normed.price as u64,
                normed.expo,
                normed.expo + PD_EXPO,
            ),
        );
        succeeds(
            pc(1, 1, 0),
            pc(i64::MAX, 1, 0),
            pc(
                (PD_SCALE as i64) / normed.price,
                PD_SCALE / (normed.price as u64),
                PD_EXPO - normed.expo,
            ),
        );

        let normed = pc(i64::MIN, u64::MAX, 0).normalize().unwrap();
        let normed_c = (-normed.price) as u64;

        succeeds(
            pc(i64::MIN, u64::MAX, 0),
            pc(i64::MIN, u64::MAX, 0),
            pc_scaled(1, 4, 0, PD_EXPO),
        );
        succeeds(
            pc(i64::MIN, u64::MAX, 0),
            pc(i64::MAX, u64::MAX, 0),
            pc_scaled(-1, 4, 0, PD_EXPO),
        );
        succeeds(
            pc(i64::MIN, u64::MAX, 0),
            pc(1, 1, 0),
            pc_scaled(
                normed.price,
                3 * normed_c,
                normed.expo,
                normed.expo + PD_EXPO,
            ),
        );
        succeeds(
            pc(1, 1, 0),
            pc(i64::MIN, u64::MAX, 0),
            pc(
                (PD_SCALE as i64) / normed.price,
                3 * (PD_SCALE / normed_c),
                PD_EXPO - normed.expo,
            ),
        );

        succeeds(
            pc(i64::MIN, 1, 0),
            pc(i64::MIN, 1, 0),
            pc_scaled(1, 0, 0, PD_EXPO),
        );
        succeeds(
            pc(i64::MIN, 1, 0),
            pc(1, 1, 0),
            pc_scaled(normed.price, normed_c, normed.expo, normed.expo + PD_EXPO),
        );
        succeeds(
            pc(1, 1, 0),
            pc(i64::MIN, 1, 0),
            pc(
                (PD_SCALE as i64) / normed.price,
                PD_SCALE / (normed_c),
                PD_EXPO - normed.expo,
            ),
        );

        // Price is zero pre-normalization
        succeeds(pc(0, 1, 0), pc(1, 1, 0), pc_scaled(0, 1, 0, PD_EXPO));
        succeeds(pc(0, 1, 0), pc(100, 1, 0), pc_scaled(0, 1, -2, PD_EXPO));
        fails(pc(1, 1, 0), pc(0, 1, 0));

        // Normalizing the input when the confidence is >> price produces a price of 0.
        fails(pc(1, 1, 0), pc(1, u64::MAX, 0));
        succeeds(
            pc(1, u64::MAX, 0),
            pc(1, 1, 0),
            pc_scaled(0, normed.conf, normed.expo, normed.expo + PD_EXPO),
        );

        // Exponent under/overflow.
        succeeds(
            pc(1, 1, i32::MAX),
            pc(1, 1, 0),
            pc(PD_SCALE as i64, 2 * PD_SCALE, i32::MAX + PD_EXPO),
        );
        fails(pc(1, 1, i32::MAX), pc(1, 1, -1));

        succeeds(
            pc(1, 1, i32::MIN - PD_EXPO),
            pc(1, 1, 0),
            pc(PD_SCALE as i64, 2 * PD_SCALE, i32::MIN),
        );
        succeeds(
            pc(1, 1, i32::MIN),
            pc(1, 1, PD_EXPO),
            pc(PD_SCALE as i64, 2 * PD_SCALE, i32::MIN),
        );
        fails(pc(1, 1, i32::MIN - PD_EXPO), pc(1, 1, 1));

        // Check timestamp will be the minimum after div
        let p1 = Price {
            publish_time: 100,
            ..pc(1234, 1234, 0)
        };

        let p2 = Price {
            publish_time: 200,
            ..pc(1234, 1234, 0)
        };

        assert_eq!(p1.div(&p2).unwrap().publish_time, 100);
        assert_eq!(p2.div(&p1).unwrap().publish_time, 100);
    }

    #[test]
    fn test_mul() {
        fn succeeds(price1: Price, price2: Price, expected: Price) {
            assert_eq!(price1.mul(&price2).unwrap(), expected);
        }

        fn fails(price1: Price, price2: Price) {
            let result = price1.mul(&price2);
            assert_eq!(result, None);
        }

        succeeds(pc(1, 1, 0), pc(1, 1, 0), pc(1, 2, 0));
        succeeds(pc(1, 1, -8), pc(1, 1, -8), pc(1, 2, -16));
        succeeds(pc(10, 1, 0), pc(1, 1, 0), pc(10, 11, 0));
        succeeds(pc(1, 1, 1), pc(1, 1, 0), pc(1, 2, 1));
        succeeds(pc(1, 1, 0), pc(5, 1, 0), pc(5, 6, 0));

        // Different exponents in the two inputs
        succeeds(pc(100, 10, -8), pc(2, 1, -7), pc(200, 120, -15));
        succeeds(pc(100, 10, -4), pc(2, 1, 0), pc(200, 120, -4));

        // Zero
        succeeds(pc(0, 10, -4), pc(2, 1, 0), pc(0, 20, -4));
        succeeds(pc(2, 1, 0), pc(0, 10, -4), pc(0, 20, -4));

        // Test with end range of possible inputs where the output should not lose precision.
        succeeds(
            pc(MAX_PD_V_I64, MAX_PD_V_U64, 0),
            pc(MAX_PD_V_I64, MAX_PD_V_U64, 0),
            pc(
                MAX_PD_V_I64 * MAX_PD_V_I64,
                2 * MAX_PD_V_U64 * MAX_PD_V_U64,
                0,
            ),
        );
        succeeds(
            pc(MAX_PD_V_I64, MAX_PD_V_U64, 0),
            pc(1, 1, 0),
            pc(MAX_PD_V_I64, 2 * MAX_PD_V_U64, 0),
        );
        succeeds(
            pc(1, MAX_PD_V_U64, 0),
            pc(3, 1, 0),
            pc(3, 1 + 3 * MAX_PD_V_U64, 0),
        );

        succeeds(
            pc(1, MAX_PD_V_U64, 0),
            pc(1, MAX_PD_V_U64, 0),
            pc(1, 2 * MAX_PD_V_U64, 0),
        );
        succeeds(
            pc(MAX_PD_V_I64, MAX_PD_V_U64, 0),
            pc(1, MAX_PD_V_U64, 0),
            pc(MAX_PD_V_I64, MAX_PD_V_U64 + MAX_PD_V_U64 * MAX_PD_V_U64, 0),
        );

        succeeds(
            pc(MIN_PD_V_I64, MAX_PD_V_U64, 0),
            pc(MIN_PD_V_I64, MAX_PD_V_U64, 0),
            pc(
                MIN_PD_V_I64 * MIN_PD_V_I64,
                2 * MAX_PD_V_U64 * MAX_PD_V_U64,
                0,
            ),
        );
        succeeds(
            pc(MIN_PD_V_I64, MAX_PD_V_U64, 0),
            pc(MAX_PD_V_I64, MAX_PD_V_U64, 0),
            pc(
                MIN_PD_V_I64 * MAX_PD_V_I64,
                2 * MAX_PD_V_U64 * MAX_PD_V_U64,
                0,
            ),
        );
        succeeds(
            pc(MIN_PD_V_I64, MAX_PD_V_U64, 0),
            pc(1, 1, 0),
            pc(MIN_PD_V_I64, 2 * MAX_PD_V_U64, 0),
        );
        succeeds(
            pc(MIN_PD_V_I64, MAX_PD_V_U64, 0),
            pc(1, MAX_PD_V_U64, 0),
            pc(MIN_PD_V_I64, MAX_PD_V_U64 + MAX_PD_V_U64 * MAX_PD_V_U64, 0),
        );

        // Unnormalized tests below here
        let ten_e7: i64 = 10000000;
        let uten_e7: u64 = 10000000;
        succeeds(
            pc(3 * (PD_SCALE as i64), 3 * PD_SCALE, PD_EXPO),
            pc(2 * (PD_SCALE as i64), 4 * PD_SCALE, PD_EXPO),
            pc(6 * ten_e7 * ten_e7, 18 * uten_e7 * uten_e7, -14),
        );

        // Test with end range of possible inputs to identify overflow
        // These inputs will lose precision due to the initial normalization.
        // Get the rounded versions of these inputs in order to compute the expected results.
        let normed = pc(i64::MAX, u64::MAX, 0).normalize().unwrap();

        succeeds(
            pc(i64::MAX, u64::MAX, 0),
            pc(i64::MAX, u64::MAX, 0),
            pc(
                normed.price * normed.price,
                4 * ((normed.price * normed.price) as u64),
                normed.expo * 2,
            ),
        );
        succeeds(
            pc(i64::MAX, u64::MAX, 0),
            pc(1, 1, 0),
            pc(normed.price, 3 * (normed.price as u64), normed.expo),
        );

        succeeds(
            pc(i64::MAX, 1, 0),
            pc(i64::MAX, 1, 0),
            pc(normed.price * normed.price, 0, normed.expo * 2),
        );
        succeeds(
            pc(i64::MAX, 1, 0),
            pc(1, 1, 0),
            pc(normed.price, normed.price as u64, normed.expo),
        );

        let normed = pc(i64::MIN, u64::MAX, 0).normalize().unwrap();
        let normed_c = (-normed.price) as u64;

        succeeds(
            pc(i64::MIN, u64::MAX, 0),
            pc(i64::MIN, u64::MAX, 0),
            pc(
                normed.price * normed.price,
                4 * (normed_c * normed_c),
                normed.expo * 2,
            ),
        );
        succeeds(
            pc(i64::MIN, u64::MAX, 0),
            pc(1, 1, 0),
            pc(normed.price, 3 * normed_c, normed.expo),
        );

        succeeds(
            pc(i64::MIN, 1, 0),
            pc(i64::MIN, 1, 0),
            pc(normed.price * normed.price, 0, normed.expo * 2),
        );
        succeeds(
            pc(i64::MIN, 1, 0),
            pc(1, 1, 0),
            pc(normed.price, normed_c, normed.expo),
        );

        // Exponent under/overflow.
        succeeds(pc(1, 1, i32::MAX), pc(1, 1, 0), pc(1, 2, i32::MAX));
        succeeds(pc(1, 1, i32::MAX), pc(1, 1, -1), pc(1, 2, i32::MAX - 1));
        fails(pc(1, 1, i32::MAX), pc(1, 1, 1));

        succeeds(pc(1, 1, i32::MIN), pc(1, 1, 0), pc(1, 2, i32::MIN));
        succeeds(pc(1, 1, i32::MIN), pc(1, 1, 1), pc(1, 2, i32::MIN + 1));
        fails(pc(1, 1, i32::MIN), pc(1, 1, -1));

        // Check timestamp will be the minimum after mul
        let p1 = Price {
            publish_time: 100,
            ..pc(1234, 1234, 0)
        };

        let p2 = Price {
            publish_time: 200,
            ..pc(1234, 1234, 0)
        };

        assert_eq!(p1.mul(&p2).unwrap().publish_time, 100);
        assert_eq!(p2.mul(&p1).unwrap().publish_time, 100);
    }
}
