# Pyth SDK Terra

This crate provides utilities for reading price feeds from the [pyth.network](https://pyth.network/) oracle on the Terra network.
The crate includes a library for reading and using Pyth data feeds in Terra.

## Usage

### Read price
 
For reading the price you just need to call `query_price` function within your contract.

```rust
// let price_id = Binary::from(b"xyz...");

// Contract address is defaulted to the mainnet address, for testnet use `testnet` feature flag.
let pyth_contract_addr = pyth_sdk_terra::CONTRACT_ADDR;

let price_feed: PriceFeed = query_price_info(deps.querier, contract_addr, price_id).unwrap().price_feed;
```

The `PriceFeed` struct contains several useful functions for working with the price.
Some of these functions are described below.
For more detailed information, please see the crate documentation.


### Get the current price

Read the current price from a `PriceFeed`: 

```rust
let current_price: PriceConf = price.get_current_price().unwrap();
println!("price: ({} +- {}) x 10^{}", current_price.price, current_price.conf, current_price.expo);
```

The price is returned along with a confidence interval that represents the degree of uncertainty in the price.
Both values are represented as fixed-point numbers, `a * 10^e`. 
The method will return `None` if the price is not currently available.

### Non-USD prices 

Most assets in Pyth are priced in USD.
Applications can combine two USD prices to price an asset in a different quote currency:

```rust
let btc_usd: Price = ...;
let eth_usd: Price = ...;
// -8 is the desired exponent for the result 
let btc_eth: Price = btc_usd.get_price_in_quote(&eth_usd, -8);
println!("BTC/ETH price: ({} +- {}) x 10^{}", price.price, price.conf, price.expo);
```

### Price a basket of assets

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

This function additionally propagates any uncertainty in the price into uncertainty in the value of the basket.

