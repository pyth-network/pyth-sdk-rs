# Pyth Network Common Rust SDK

This crate contains Pyth Network data structures that are shared across all Rust-based consumers of Pyth Network data.
This crate is typically used in combination with a platform-specific crate such as [pyth-sdk-solana](../pyth-sdk-solana) or [pyth-sdk-terra](../pyth-sdk-terra).

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
let current_price: Price = price_feed.get_current_price().ok_or(StdError::not_found("Current price is not available"))?;
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
let ema_price: Price = price_feed.get_ema_price().ok_or(StdError::not_found("EMA price is not available"))?;
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
