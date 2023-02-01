use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use std::convert::TryFrom;

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

    /// Get the valuation of a collateral position according to:
    /// 1. the net amount currently deposited (across the protocol)
    /// 2. the deposits endpoint for the affine combination (across the protocol)
    /// 3. the initial (at 0 deposits) and final (at the deposits endpoint) valuation discount rates
    /// 
    /// We use a linear interpolation between the the initial and final discount rates,
    /// scaled by the proportion of the deposits endpoint that has been deposited.
    /// This essentially assumes a linear liquidity cumulative density function,
    /// which has been shown to be a reasonable assumption for many crypto tokens in literature.
    /// For more detail on this: https://pythnetwork.medium.com/improving-lending-protocols-with-liquidity-oracles-fd1ea4f96f37
    /// 
    /// If the assumptions of the liquidity curve hold true, we are obtaining a lower bound for the net price
    /// at which one can sell the quantity of token specified by deposits in the open markets.
    /// We value collateral according to the total deposits in the protocol due to the present
    /// intractability of assessing collateral at risk by price range.
    /// 
    /// Args
    /// deposits: u64, quantity of token deposited in the protocol
    /// deposits_endpoint: u64, deposits right endpoint for the affine combination
    /// discount_initial: u64, initial discount rate at 0 deposits (units given by discount_exponent)
    /// discount_final: u64, final discount rate at deposits_endpoint deposits (units given by discount_exponent)
    /// discount_exponent: u64, the exponent to apply to the discounts above (e.g. if discount_final is 10 but meant to express 0.1/10%, exponent would be -2)
    /// note that if discount_initial is bigger than 100% per the discount exponent scale, then the initial valuation of the collateral will be higher than the oracle price
    /// 
    /// affine_combination yields us error <= 2/PD_SCALE for discount_interpolated
    /// We then multiply this with the price to yield price_discounted before scaling this back to the original expo
    /// Output of affine_combination has expo >= -18, price (self) has arbitrary expo
    /// Scaling this back to the original expo then has error bounded by the expo (10^expo).
    /// This is because reverting a potentially finer expo to a coarser grid has the potential to be off by
    /// the order of the atomic unit of the coarser grid.
    /// This scaling error combines with the previous error additively: Err <= 2/PD_SCALE + 10^expo
    /// 
    /// The practical error is based on the original expo:
    /// if it is big, then the 10^expo loss dominates;
    /// otherwise, the 2/PD_SCALE error dominates.
    /// 
    /// Thus, we expect the computed collateral valuation price to be no more than 2/PD_SCALE + 10^expo off of the mathematically true value
    /// For this reason, we encourage using this function with prices that have high expos, to minimize the potential error.
    pub fn get_collateral_valuation_price(&self, deposits: u64, deposits_endpoint: u64, discount_initial: u64, discount_final: u64, discount_exponent: i32) -> Option<Price> {
        if discount_initial < discount_final {
            return None;
        }

        // get price versions of discounts
        let initial_percentage = Price {
            price: i64::try_from(discount_initial).ok()?,
            conf: 0,
            expo: discount_exponent,
            publish_time: 0,
        };
        let final_percentage = Price {
            price: i64::try_from(discount_final).ok()?,
            conf: 0,
            expo: discount_exponent,
            publish_time: 0,
        };

        // get the interpolated discount as a price
        let discount_interpolated = Price::affine_combination(
            0, 
            initial_percentage, 
            i64::try_from(deposits_endpoint).ok()?, 
            final_percentage, 
            i64::try_from(deposits).ok()?,
            -9
        )?;

        let conf_orig = self.conf;
        let expo_orig = self.expo;

        // get price discounted, convert back to the original exponents we received the price in
        let price_discounted = self.
            mul(&discount_interpolated)?.
            scale_to_exponent(expo_orig)?
        ;
        
        return Some(
            Price {
                price: price_discounted.price,
                conf: conf_orig,
                expo: price_discounted.expo,
                publish_time: self.publish_time,
            }
        );
    }

    /// Get the valuation of a borrow position according to:
    /// 1. the net amount currently borrowed (across the protocol)
    /// 2. the borrowed endpoint for the affine combination (across the protocol)
    /// 3. the initial (at 0 borrows) and final (at the borrow endpoint) valuation premiums
    /// 
    /// We use a linear interpolation between the the initial and final premiums,
    /// scaled by the proportion of the borrows endpoint that has been borrowed out.
    /// This essentially assumes a linear liquidity cumulative density function,
    /// which has been shown to be a reasonable assumption for many crypto tokens in literature.
    /// For more detail on this: https://pythnetwork.medium.com/improving-lending-protocols-with-liquidity-oracles-fd1ea4f96f37
    /// 
    /// If the assumptions of the liquidity curve hold true, we are obtaining an upper bound for the net price
    /// at which one can buy the quantity of token specified by borrows in the open markets.
    /// We value the borrows according to the total borrows out of the protocol due to the present
    /// intractability of assessing collateral at risk and repayment likelihood by price range.
    /// 
    /// Args
    /// borrows: u64, quantity of token borrowed from the protocol
    /// borrows_endpoint: u64, borrows right endpoint for the affine combination
    /// premium_initial: u64, initial premium at 0 borrows (units given by premium_exponent)
    /// premium_final: u64, final premium at borrows_endpoint borrows (units given by premium_exponent)
    /// premium_exponent: u64, the exponent to apply to the premiums above (e.g. if premium_final is 50 but meant to express 0.05/5%, exponent would be -3)
    /// note that if premium_initial is less than 100% per the premium exponent scale, then the initial valuation of the borrow will be lower than the oracle price
    /// 
    /// affine_combination yields us error <= 2/PD_SCALE for premium_interpolated
    /// We then multiply this with the price to yield price_premium before scaling this back to the original expo
    /// Output of affine_combination has expo >= -18, price (self) has arbitrary expo
    /// Scaling this back to the original expo then has error bounded by the expo (10^expo).
    /// This is because reverting a potentially finer expo to a coarser grid has the potential to be off by
    /// the order of the atomic unit of the coarser grid.
    /// This scaling error combines with the previous error additively: Err <= 2/PD_SCALE + 10^expo
    /// 
    /// The practical error is based on the original expo:
    /// if it is big, then the 10^expo loss dominates;
    /// otherwise, the 2/PD_SCALE error dominates.
    /// 
    /// Thus, we expect the computed borrow valuation price to be no more than 2/PD_SCALE + 10^expo off of the mathematically true value
    /// For this reason, we encourage using this function with prices that have high expos, to minimize the potential error.
    pub fn get_borrow_valuation_price(&self, borrows: u64, borrows_endpoint: u64, premium_initial: u64, premium_final: u64, premium_exponent: i32) -> Option<Price> {
        if premium_initial > premium_final {
            return None;
        }

        // get price versions of premiums
        let initial_percentage = Price {
            price: i64::try_from(premium_initial).ok()?,
            conf: 0,
            expo: premium_exponent,
            publish_time: 0,
        };
        let final_percentage = Price {
            price: i64::try_from(premium_final).ok()?,
            conf: 0,
            expo: premium_exponent,
            publish_time: 0,
        };

        // get the interpolated premium as a price
        let premium_interpolated = Price::affine_combination(
            0,
            initial_percentage, 
            i64::try_from(borrows_endpoint).ok()?, 
            final_percentage, 
            i64::try_from(borrows).ok()?,
            -9
        )?;

        let conf_orig = self.conf;
        let expo_orig = self.expo;

        // get price premium, convert back to the original exponents we received the price in
        let price_premium = self.
            mul(&premium_interpolated)?.
            scale_to_exponent(expo_orig)?
        ;

        println!("init perc: {}, {}", initial_percentage.price, initial_percentage.expo);
        println!("final perc: {}, {}", final_percentage.price, final_percentage.expo);
        println!("interpolated premium: {}, {}", premium_interpolated.price, premium_interpolated.expo);
        println!("in btwn: {}, {}", self.mul(&premium_interpolated)?.price, self.mul(&premium_interpolated)?.expo);
        println!("price (adj for premium): {}, {}", price_premium.price, price_premium.expo);
        println!("=======");
        
        return Some(
            Price {
                price: price_premium.price,
                conf: conf_orig,
                expo: price_premium.expo,
                publish_time: self.publish_time,
            }
        );
    }

    /// Performs an affine combination after setting everything to expo -9
    /// Takes in 2 points and a 3rd "query" x coordinate, to compute the value at
    /// Effectively draws a line between the 2 points and then proceeds to 
    /// interpolate/exterpolate to find the value at the query coordinate according to that line
    /// 
    /// Args
    /// x1: i64, the x coordinate of the first point
    /// y1: Price, the y coordinate of the first point, represented as a Price struct
    /// x2: i64, the x coordinate of the second point, must be greater than x1
    /// y2: Price, the y coordinate of the second point, represented as a Price struct
    /// x_query: i64, the query x coordinate, at which we wish to impute a y value
    /// pre_add_expo: i32, the exponent to scale to, before final addition; essentially the final precision you want
    /// 
    /// Logic
    /// imputed y value = y2 * ((xq-x1)/(x2-x1)) + y1 * ((x2-x3)/(x2-x1))
    /// 1. compute A = xq-x1
    /// 2. compute B = x2-xq
    /// 3. compute C = x2-x1
    /// 4. compute D = A/C
    /// 5. compute E = B/C
    /// 6. compute F = y2 * D
    /// 7. compute G = y1 * E
    /// 8. compute H = F + G
    /// 
    /// Bounds due to precision loss
    /// x = 1/PD_SCALE
    /// division incurs max loss of x
    /// Err(D), Err(E) is relatively negligible--by scaling to expo -9 we are imposing
    /// a grid composed of 1 billion units between x1 and x2 endpoints. Moreover, D, E <= 1.
    /// Thus, max loss here: Err(D), Err(E) <= x
    /// Err(y1), Err(y2) with normalization <= x
    /// Err(F), Err(G) <= (1+x)^2 - 1 (in fractional terms)
    /// Err(H) <= 2*(1+x)^2 - 2 ~= 2x = 2/PD_SCALE
    /// 
    /// Scaling this back has error bounded by the expo (10^pre_add_expo).
    /// This is because reverting a potentially finer expo to a coarser grid has the potential to be off by
    /// the order of the atomic unit of the coarser grid.
    /// This scaling error combines with the previous error additively: Err <= 2/PD_SCALE + 2*10^pre_add_expo
    /// But if pre_add_expo is reasonably small (<= -9), then other term will dominate
    pub fn affine_combination(x1: i64, y1: Price, x2: i64, y2: Price, x_query: i64, pre_add_expo: i32) -> Option<Price> {
        if x2 <= x1 {
            return None;
        }
        
        // get the deltas for the x coordinates
        // 1. compute A = xq-x1
        let delta_q1 = x_query.checked_sub(x1)?;
        // 2. compute B = x2-xq
        let delta_2q = x2.checked_sub(x_query)?;
        // 3. compute C = x2-x1
        let delta_21 = x2.checked_sub(x1)?;

        // get the relevant fractions of the deltas, with scaling
        // 4. compute D = A/C, Err(D) <= x = 1/PD_SCALE
        let frac_q1 = Price::fraction(delta_q1, delta_21)?;
        // 5. compute E = B/C, Err(E) <= x
        let frac_2q = Price::fraction(delta_2q, delta_21)?;

        // calculate products for left and right
        // 6. compute F = y2 * D, Err(F) <= (1+x)^2 - 1
        let mut left = y2.mul(&frac_q1)?;
        // 7. compute G = y1 * E, Err(G) <= (1+x)^2 - 1
        let mut right = y1.mul(&frac_2q)?; 

        println!("frac_q1: {}, {}", frac_q1.price, frac_q1.expo);
        println!("frac_2q: {}, {}", frac_2q.price, frac_2q.expo);

        println!("left before scaling: {}, {}", left.price, left.expo);
        println!("right before scaling: {}, {}", right.price, right.expo);

        // Err(scaling) += 2*10^pre_add_expo
        left = left.scale_to_exponent(pre_add_expo)?;
        right = right.scale_to_exponent(pre_add_expo)?;
        
        if left.expo != right.expo {
            return None;
        }

        println!("left after scaling: {}, {}", left.price, left.expo);
        println!("right after scaling: {}, {}", right.price, right.expo);

        // 8. compute H = F + G, Err(H) ~= 2x + 2*10^pre_add_expo
        return left.add(&right);
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
        println!("numerator orig: {}, {}", self.price, self.expo);
        println!("orig price other: {}, {}", other.price, other.expo);

        let base = self.normalize()?;
        let other = other.normalize()?;

        println!("numerator post: {}, {}", base.price, base.expo);
        println!("orig price post: {}, {}", other.price, other.expo);

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

    /// Helper function to check if precision implied by exponent >= value, exponent <= 0
    fn check_precision_geq(value: u64, exponent: i32) -> Option<bool> {  
        if exponent > 0 {
            return None;
        }
        
        let mut precision: u64 = 1;
        let mut delta = -exponent;
        
        while delta > 0 && (value != 0) {
            precision = precision.checked_mul(10)?;
            delta = delta.checked_sub(1)?;
        }

        return Some(value <= precision);
    }

    /// Helper function to create fraction
    /// 
    /// fraction(x, y) gives you the normalized Price closest to x/y.
    /// If you cannot represent x/y exactly within 8 digits of precision, it may zero out the remainder.
    /// In particular, if x and/or y cannot be represented within 8 digits of precision, potential for precision error.
    /// If x and y can both be represented within 8 digits of precision AND x/y can be represented within 8 digits, no precision loss.
    /// 
    /// Error of normalizing x, y <= 10^(PD_EXPO+2) = 10^-7
    /// Inherits any bounded errors from normalization and div
    fn fraction(x: i64, y: i64) -> Option<Price> {
        // convert x and y to Prices
        let x_as_price = Price {
            price: x,
            conf: 0,
            expo: 0,
            publish_time: 0,
        };
        let y_as_price = Price {
            price: y,
            conf: 0,
            expo: 0,
            publish_time: 0,
        };

        // get the relevant fraction
        let mut frac = x_as_price.div(&y_as_price)?;

        return Some(frac);
    }
}

#[cfg(test)]
mod test {
    use quickcheck::{Arbitrary, Gen, TestResult};

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

    // arbitrary trait for quickcheck
    impl Arbitrary for Price {
        fn arbitrary(g: &mut Gen) -> Price {
            Price {
                price: i64::arbitrary(g),
                conf: u64::arbitrary(g),
                expo: i32::arbitrary(g), 
                publish_time: 0
            }
        }
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

    #[test]
    fn test_check_precision_geq() {
        fn succeeds(value: u64, exponent: i32, expected: bool) {
            let precision_geq = Price::check_precision_geq(value, exponent).unwrap();

            assert_eq!(precision_geq, expected);
        }

        fn fails(value: u64, exponent: i32) {
            let result = Price::check_precision_geq(value, exponent);
            assert_eq!(result, None);
        }

        succeeds(1, 0, true);
        succeeds(1, -1, true);

        succeeds(100, -1, false);
        succeeds(100, -2, true);
        succeeds(100, -3, true);

        succeeds(101, -2, false);
        succeeds(101, -3, true);

        // fails bc exponent > 0
        fails(10, 1);
    }

    #[test]
    fn test_get_collateral_valuation_price() {
        fn succeeds(price: Price, deposits: u64, deposits_endpoint: u64, discount_initial: u64, discount_final: u64, discount_exponent: i32, expected: Price) {
            let price_collat = price.get_collateral_valuation_price(deposits, deposits_endpoint, discount_initial, discount_final, discount_exponent).unwrap();

            assert_eq!(price_collat, expected);
        }

        fn fails(price: Price, deposits: u64, deposits_endpoint: u64, discount_initial: u64, discount_final: u64, discount_exponent: i32) {
            let result = price.get_collateral_valuation_price(deposits, deposits_endpoint, discount_initial, discount_final, discount_exponent);
            assert_eq!(result, None);
        }

        // 0 deposits
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            0,
            100,
            100,
            90,
            -2,
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // half deposits
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            50,
            100,
            100,
            90,
            -2,
            pc(95 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // full deposits
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100,
            100,
            100,
            90,
            -2,
            pc(90 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // 0 deposits, diff precision
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            0,
            100,
            1000,
            900,
            -3,
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // half deposits, diff precision
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            50,
            100,
            1000,
            900,
            -3,
            pc(95 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // full deposits, diff precision
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100,
            100,
            1000,
            900,
            -3,
            pc(90 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // beyond final endpoint deposits
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            150,
            100,
            100,
            90,
            -2,
            pc(85 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // 0 deposits, staggered initial discount
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            0,
            100,
            98,
            90,
            -2,
            pc(98 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // half deposits, staggered initial discount
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            50,
            100,
            98,
            90,
            -2,
            pc(94 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // full deposits, staggered initial discount
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100,
            100,
            98,
            90,
            -2,
            pc(90 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // test precision limits
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            0,
            1_000_000_000_000_000_000,
            100,
            90,
            -2,
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            1,
            1_000_000_000_000_000_000,
            100,
            90,
            -2,
            pc(100 * (PD_SCALE as i64)-1000, 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100_000_000,
            1_000_000_000_000_000_000,
            100,
            90,
            -2,
            pc(100 * (PD_SCALE as i64)-1000, 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            1_000_000_000,
            1_000_000_000_000_000_000,
            100,
            90,
            -2,
            pc(100 * (PD_SCALE as i64)-1000, 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            10_000_000_000,
            1_000_000_000_000_000_000,
            100,
            90,
            -2,
            pc(100 * (PD_SCALE as i64)-1000, 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100_000_000_000,
            1_000_000_000_000_000_000,
            100,
            90,
            -2,
            pc(100 * (PD_SCALE as i64)-1000, 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            200_000_000_000,
            1_000_000_000_000_000_000,
            100,
            90,
            -2,
            pc(100 * (PD_SCALE as i64)-2000, 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            1_000_000_000_000,
            1_000_000_000_000_000_000,
            100,
            90,
            -2,
            pc(100 * (PD_SCALE as i64)-10000, 2 * PD_SCALE, -9),
        );

        // fails bc initial discount lower than final discount
        fails(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            50,
            100,
            89,
            90,
            -2
        );
    }

    #[test]
    fn test_get_borrow_valuation_price() {
        fn succeeds(price: Price, borrows: u64, borrows_endpoint: u64, premium_initial: u64, premium_final: u64, premium_exponent: i32, expected: Price) {
            let price_borrow = price.get_borrow_valuation_price(borrows, borrows_endpoint, premium_initial, premium_final, premium_exponent).unwrap();
            
            assert_eq!(price_borrow, expected);
        }

        fn fails(price: Price, borrows: u64, borrows_endpoint: u64, premium_initial: u64, premium_final: u64, premium_exponent: i32) {
            let result = price.get_borrow_valuation_price(borrows, borrows_endpoint, premium_initial, premium_final, premium_exponent);
            assert_eq!(result, None);
        }

        // 0 borrows
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            0,
            100,
            100,
            110,
            -2,
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // half borrows
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            50,
            100,
            100,
            110,
            -2,
            pc(105 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // full borrows
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100,
            100,
            100,
            110,
            -2,
            pc(110 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // 0 borrows, diff precision
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            0,
            100,
            1000,
            1100,
            -3,
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // half borrows, diff precision
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            50,
            100,
            1000,
            1100,
            -3,
            pc(105 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // full borrows, diff precision
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100,
            100,
            1000,
            1100,
            -3,
            pc(110 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // beyond final endpoint borrows
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            150,
            100,
            100,
            110,
            -2,
            pc(115 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // 0 borrows, staggered initial premium
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            0,
            100,
            102,
            110,
            -2,
            pc(102 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // half borrows, staggered initial premium
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            50,
            100,
            102,
            110,
            -2,
            pc(106 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );

        // full borrows, staggered initial premium
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100,
            100,
            102,
            110,
            -2,
            pc(110 * (PD_SCALE as i64), 2 * PD_SCALE, -9)
        );






        

        // test precision limits
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            1,
            1_000_000_000_000_000_000,
            100,
            110,
            -2,
            pc(100 * (PD_SCALE as i64 - 10), 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100_000_000,
            1_000_000_000_000_000_000,
            100,
            110,
            -2,
            pc(100 * (PD_SCALE as i64 - 10), 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            1_000_000_000,
            1_000_000_000_000_000_000,
            100,
            110,
            -2,
            pc(100 * (PD_SCALE as i64 - 10), 2 * PD_SCALE, -9),
        );
        // interpolation now doesn't lose precision, but normalize in final multiply loses precision
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            10_000_000_000,
            1_000_000_000_000_000_000,
            100,
            110,
            -2,
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            20_000_000_000,
            1_000_000_000_000_000_000,
            100,
            110,
            -2,
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
        );
        // precision no longer lost
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            100_000_000_000,
            1_000_000_000_000_000_000,
            100,
            110,
            -2,
            pc(100 * (PD_SCALE as i64 + 10), 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            200_000_000_000,
            1_000_000_000_000_000_000,
            100,
            110,
            -2,
            pc(100 * (PD_SCALE as i64 + 20), 2 * PD_SCALE, -9),
        );
        succeeds(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            1_000_000_000_000,
            1_000_000_000_000_000_000,
            100,
            110,
            -2,
            pc(100 * (PD_SCALE as i64+100), 2 * PD_SCALE, -9),
        );

        // fails bc initial premium exceeds final premium
        fails(
            pc(100 * (PD_SCALE as i64), 2 * PD_SCALE, -9),
            50,
            100,
            111,
            110,
            -2
        );

    }

    #[test]
    fn test_affine_combination() {
        fn succeeds(x1: i64, y1: Price, x2: i64, y2: Price, x_query: i64, pre_add_expo: i32, expected: Price) {
            let y_query = Price::affine_combination(x1, y1, x2, y2, x_query, pre_add_expo).unwrap();
            
            assert_eq!(y_query, expected);
        }

        fn fails(x1: i64, y1: Price, x2: i64, y2: Price, x_query: i64, pre_add_expo: i32) {
            let result = Price::affine_combination(x1, y1, x2, y2, x_query, pre_add_expo);
            assert_eq!(result, None);
        }

        // constant, inbounds
        succeeds(
            0,
            pc(100, 0, -4),
            10,
            pc(100, 0, -4),
            5,
            -9,
            pc(10_000_000, 0, -9)
        );

        // constant, outside the bounds
        succeeds(
            0,
            pc(100, 0, -4),
            10,
            pc(100, 0, -4),
            15,
            -9,
            pc(10_000_000, 0, -9),
        );

        // increasing, inbounds
        succeeds(
            0,
            pc(90, 0, -4),
            10,
            pc(100, 0, -4),
            5,
            -9,
            pc(9_500_000, 0, -9)
        );

        // increasing, out of bounds
        succeeds(
            0,
            pc(90, 0, -4),
            10,
            pc(100, 0, -4),
            15,
            -9,
            pc(10_500_000, 0, -9)
        );

        // decreasing, inbounds
        succeeds(
            0,
            pc(100, 0, -4),
            10,
            pc(80, 0, -4),
            5,
            -9,
            pc(9_000_000, 0, -9)
        );

        // decreasing, out of bounds
        succeeds(
            0,
            pc(100, 0, -4),
            10,
            pc(80, 0, -4),
            15,
            -9,
            pc(7_000_000, 0, -9)
        );

        // test loss due to scaling
        succeeds(
            0,
            pc(0, 0, -2),
            13,
            pc(10, 0, -2),
            1,
            -8,
            pc(769230, 0, -8)
        );
        succeeds(
            0,
            pc(0, 0, -2),
            13,
            pc(10, 0, -2),
            1,
            -9,
            pc(7692307, 0, -9)
        );
        succeeds(
            0,
            pc(0, 0, -3),
            13,
            pc(100, 0, -3),
            1,
            -9,
            pc(7692307, 0, -9)
        );
        succeeds(
            0,
            pc(0, 0, -2),
            13,
            pc(100, 0, -2),
            1,
            -9,
            pc(76923076, 0, -9)
        );

        // Test with end range of possible inputs on endpoint xs (no precision loss)
        succeeds(
            0,
            pc(100, 0, -9),
            i64::MAX,
            pc(0, 0, -9),
            i64::MAX/10,
            -9,
            pc(90, 0, -9)
        );
        succeeds(
            i64::MIN,
            pc(100, 0, -9),
            i64::MIN/2,
            pc(0, 0, -9),
            (i64::MIN/4)*3,
            -9,
            pc(50, 0, -9)
        );
        succeeds(
            i64::MIN+1,
            pc(100, 0, -9),
            0,
            pc(0, 0, -9),
            i64::MIN/4,
            -9,
            pc(25, 0, -9)
        );
        succeeds(
            i64::MIN+1,
            pc(100, 0, -9),
            0,
            pc(0, 0, -9),
            0,
            -9,
            pc(0, 0, -9)
        );

        // Test with end range of possible inputs in prices to identify precision inaccuracy
        succeeds(
            0,
            pc(MAX_PD_V_I64-10, 0, -4),
            10,
            pc(MAX_PD_V_I64, 0, -4),
            5,
            -4,
            pc(MAX_PD_V_I64-6, 0, -4)
        );
        succeeds(
            0,
            pc(MAX_PD_V_I64-1, 0, -4),
            10,
            pc(MAX_PD_V_I64, 0, -4),
            9,
            -4,
            pc(MAX_PD_V_I64-1, 0, -4)
        );

        // test w confidence (same at both endpoints)
        succeeds(
            0,
            pc(90, 10, -4),
            10,
            pc(100, 10, -4),
            5,
            -9,
            pc(9_500_000, 1_000_000, -9)
        );

        // test w confidence (different at the endpoints)
        succeeds(
            0,
            pc(90, 10, -4),
            10,
            pc(100, 15, -4),
            5,
            -9,
            pc(9_500_000, 1_250_000, -9)
        );
        succeeds(
            0,
            pc(90, 10, -4),
            10,
            pc(100, 15, -4),
            8,
            -9,
            pc(9_800_000, 1_400_000, -9)
        );
        succeeds(
            0,
            pc(90, 10, -4),
            10,
            pc(100, 15, -4),
            15,
            -9,
            pc(10_500_000, 2_750_000, -9)
        );

        // fails bc x1 > x2
        fails(
            20,
            pc(100, 0, -4),
            10,
            pc(100, 0, -4),
            15,
            -9
        );
        // fails bc x1 is MIN, x2-x1 --> overflow in delta
        fails(
            i64::MIN,
            pc(100, 0, -5),
            10,
            pc(1000, 0, -5),
            5,
            -9
        );
        // fails bc x2 is MAX, x1 is negative --> overflow in delta
        fails(
            -5,
            pc(100, 0, -4),
            i64::MAX,
            pc(1000, 0, -4),
            5,
            -9
        );
        // fails bc of overflow in the checked_sub
        fails(
            i64::MIN/2,
            pc(100, 0, -4),
            i64::MAX/2+1,
            pc(100, 0, -4),
            5,
            -9
        );
        // fails bc price too small to be realized
        fails(
            0,
            pc(100, 0, -4),
            10,
            pc(5, 0, -4),
            i64::MAX-100,
            -9
        );
        // fails bc 0-i64::MIN > i64::MAX, so overflow
        fails(
            i64::MIN,
            pc(100, 0, -9),
            0,
            pc(0, 0, -9),
            0,
            -9
        );
    }

    // quickcheck to confirm affine_combination introduces no error if normalization done explicitly on prices
    #[quickcheck]
        fn quickcheck_affine(x1: i64, y1: Price, x2: i64, y2: Price, x_query: i64, pre_add_expo: i32) -> TestResult {
            // require x2 > x1
            if x1 >= x2 {
                return TestResult::discard()
            }

            // require low pre_add_expo
            if pre_add_expo >= 2 {
                return TestResult::discard()
            }

            // require reasonable price/conf range
            if (y1.price > 2*MAX_PD_V_I64) || (y1.price < 2*MIN_PD_V_I64) || (y1.conf > 2*MAX_PD_V_U64) {
                return TestResult::discard()
            }
            if (y2.price > 2*MAX_PD_V_I64) || (y2.price < 2*MIN_PD_V_I64) || (y2.conf > 2*MAX_PD_V_U64) {
                return TestResult::discard()
            }

            let result_orig = Price::affine_combination(x1, y1, x2, y2, x_query, pre_add_expo).unwrap();

            let y1_norm = y1.normalize().unwrap();
            let y2_norm = y2.normalize().unwrap();

            let result_norm = Price::affine_combination(x1, y1_norm, x2, y2_norm, x_query, pre_add_expo).unwrap();

            TestResult::from_bool(result_norm == result_orig)
        }

    #[test]
    fn test_fraction() {
        fn succeeds(x: i64, y: i64, expected: Price) {
            let frac = Price::fraction(x, y).unwrap();
            
            assert_eq!(frac, expected);
        }

        fn fails(x: i64, y: i64) {
            let result = Price::fraction(x, y);

            assert_eq!(result, None);
        }
    
        // check basic tests of fraction division
        succeeds(
            100, 
            1000,
            pc(100_000_000, 0, -9)
        );
        succeeds(
            1,
            1_000_000_000,
            pc(10, 0, -10)
        );
        // when x and y and x/y can be represented in 8 digits, no loss
        succeeds(
            10_000_001,
            20_000_002,
            pc(500_000_000, 0, -9)
        );
        succeeds(
            102,
            3,
            pc(34_000_000_000, 0, -9)
        );

        // test loss due to big numer (x cannot be represented in 8 digits)--only preserves 8 digits of precision
        succeeds(
            3_000_000_021_123,
            1,
            pc(30_000_000_000_000_000, 0, -4)
        );

        // test loss due to big denom (y cannot be represented in 8 digits)
        succeeds(
            1,
            10_000_000_011,
            pc(10, 0, -11)
        );

        // x and y representable within 8 digits, but x/y is not
        succeeds(
            1,
            7,
            pc(142_857_142, 0, -9)
        );

        // Test with big inputs where the output will lose precision.
        // x not representable within 8 digits
        succeeds(
            i64::MAX,
            100,
            pc(922337200000000, 0, 2)
        );
        succeeds(
            i64::MAX,
            1,
            pc(92233720000000000, 0, 2)
        );
        // Neither x nor y representable within 8 digits
        succeeds(
            i64::MAX-10,
            i64::MAX-10_000_000_000,
            pc(1000000000, 0, -9)
        );
        // Neither x nor y representable within 8 digits, but this subtraction actually influences relevant digit for precision
        succeeds(
            i64::MAX-10,
            i64::MAX-100_000_000_000,
            pc(1_000_000_010, 0, -9)
        );

        // Test with end range of possible inputs where the output should not lose precision.
        succeeds(
            MAX_PD_V_I64,
            MAX_PD_V_I64,
            pc(1_000_000_000, 0, -9)
        );
        succeeds(
            MAX_PD_V_I64,
            1,
            pc(MAX_PD_V_I64*1_000_000_000, 0, -9)
        );
        succeeds(
            MAX_PD_V_I64,
            MIN_PD_V_I64,
            pc(-1_000_000_000, 0, -9)
        );
        succeeds(
            MIN_PD_V_I64,
            1,
            pc(MIN_PD_V_I64*1_000_000_000, 0, -9)
        );

        // fails due to div by 0
        fails(
            100,
            0,
        );
    }
}
