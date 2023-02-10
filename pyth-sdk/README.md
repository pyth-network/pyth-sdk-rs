# Pyth Network Common Rust SDK

This crate contains Pyth Network data structures that are shared across all Rust-based consumers of Pyth Network data.
This crate is typically used in combination with a platform-specific crate such as [pyth-sdk-solana](../pyth-sdk-solana).

## Usage

The SDK has two core data types:

* `PriceFeed` is a container for all currently-available pricing information about a product (e.g., BTC/USD).
* `Price` represents a price with a degree of uncertainty.

The typical usage of this SDK is to first retrieve a `PriceFeed` for one or more products required by your application.
This step typically uses one of the platform-specific crates (referenced above), which provide retrieval methods for specific blockchains.
Once you have a `PriceFeed`, you can call one of the methods below to get the prices your application needs:

### Get the Current Price

Get the current price of the product from its `PriceFeed`:

```rust
const STALENESS_THRESHOLD : u64 = 60; // staleness threshold in seconds
let current_timestamp = ...;
let current_price: Price = price_feed.get_price_no_older_than(current_timestamp, STALENESS_THRESHOLD).ok_or(StdError::not_found("Current price is not available"))?;
println!("price: ({} +- {}) x 10^{}", current_price.price, current_price.conf, current_price.expo);
```

The price is returned along with a confidence interval that represents the degree of uncertainty in the price.
Both values are represented as fixed-point numbers, `a * 10^e`.
The method will return `None` if the current price is not available.

Please see the [consumer best practices guide](https://docs.pyth.network/consumers/best-practices) for additional recommendations on how to consume Pyth Network prices, such as how to use the confidence interval, and what to do if the price is not currently available.

### EMA Price

`PriceFeed` includes an exponentially-weighted moving average (EMA) price that represents a time-average of recent prices.
The EMA price can be retrieved as follows:

```rust
const STALENESS_THRESHOLD : u64 = 60; // staleness threshold in seconds
let current_timestamp = ...;
let ema_price: Price = price_feed.get_ema_price_no_older_than(current_timestamp, STALENESS_THRESHOLD).ok_or(StdError::not_found("EMA price is not available"))?;
println!("price: ({} +- {}) x 10^{}", ema_price.price, ema_price.conf, ema_price.expo);
```

## Manipulating Prices

The `Price` struct supports arithmetic operations that allow you to combine prices from multiple products.
These operations can be used to price some products that aren't directly listed on Pyth Network:

### Non-USD Prices

Most assets listed on Pyth Network are quoted in terms of USD, e.g., the BTC/USD price feed provides the number of dollars per BTC.
However, some applications would like prices in other quote currencies, such as the number of ETH per BTC.
Applications can combine two USD prices to price an asset in a different quote currency:

```rust
let btc_usd: Price = ...;
let eth_usd: Price = ...;
// -8 is the desired exponent for the result
let btc_eth: Price = btc_usd.get_price_in_quote(&eth_usd, -8);
println!("BTC/ETH price: ({} +- {}) x 10^{}", price.price, price.conf, price.expo);
```

### Price a Basket of Assets

Applications can also compute the value of a basket of multiple assets:

```rust
let btc_usd: Price = ...;
let eth_usd: Price = ...;
// Quantity of each asset in fixed-point a * 10^e.
// This represents 0.1 BTC and .05 ETH.
// -8 is desired exponent for result
let basket_price: Price = Price::price_basket(&[
    (btc_usd, 10, -2),
    (eth_usd, 5, -2)
  ], -8);
println!("0.1 BTC and 0.05 ETH are worth: ({} +- {}) x 10^{} USD",
         basket_price.price, basket_price.conf, basket_price.expo);
```

This operation can be useful for pricing, e.g., an LP token that is backed by two underlying currencies.

## Adjust Prices using Liquidity

Applications can adjust Pyth prices to incorporate market impact and liquidity for large positions, since the effective transaction price for large positions differs from the midprice and top-of-book price. Suppose the application wants to value the effective execution price on selling 100 BTC, based on the fact that 1000 BTC sell at a 10% discount (90% of the midprice). Based on assuming the market impact is constant with respect to amount sold, the application can combine the current Pyth price and its liquidity views to calculate the adjusted price:

```rust
let btc_usd: Price = ...;
// deposits is the total number of tokens deposited in the protocol
let deposits: u64 = 100;
// deposits_endpoint represents the token deposits at which rate_discount_final kicks in
let deposits_endpoint: u64 = 1000;
// rate_discount_initial and _final are percentages, expressed in fixed point as x * 10^discount_exponent.
// E.g. 50 with discount_exponent = -2 is 50%.
let rate_discount_initial: u64 = 100;
let rate_discount_final: u64 = 90;
let discount_exponent: i32 = 2;

let price_collateral: Price = btc_usd.get_collateral_valuation_price(
    deposits,
    deposits_endpoint,
    rate_discount_initial,
    rate_discount_final,
    discount_exponent).ok_or(StdError::not_found("Issue with querying collateral price"))?;
println!("The valuation price for the collateral given {} tokens deposited is ({} +- {}) x 10^{} USD",
         deposits, price_collateral.price, price_collateral.conf, price_collateral.expo);
```

Here, `deposits` indicates the total amount of collateral deposited. `get_collateral_valuation_price` takes in the total deposits in the protocol and linearly interpolates between (`0`, `rate_discount_inital`) and (`deposits_endpoint`, `rate_discount_final`) to calculate the discount at `deposits`. As a note, this function scales the price depending on the provided discount and deposit inputs, but it does not alter the confidence.

To adjust the price at which a borrow position is valued, a protocol can similarly combine the current Pyth price and their estimate of liquidity, but using the `get_borrow_valuation_price` function now in place of `get_collateral_valuation_price`.
