# Pyth SDK Terra

This crate provides utilities for reading price feeds from the [pyth.network](https://pyth.network/) oracle on the Terra network.
The crate includes a library for reading and using Pyth data feeds in Terra.

## Usage

### Read price

For reading the price you just need to call `query_price_feed` function within your contract with the id of the price.

You can find the contract address and price ids in the section [Contracts and Price Feeds](#contracts-and-price-feeds) below.

```rust
let price_feed: PriceFeed = query_price_feed(deps.querier, contract_addr, price_id).unwrap().price_feed;
```

The `PriceFeed` struct contains several useful functions for working with the price.
Some of these functions are described below.
For more detailed information, please see the crate documentation.


### Get the current price

Read the current price from a `PriceFeed`: 

```rust
let current_price: Price = price.get_current_price().unwrap();
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

## Contracts and Price Feeds

Currently Pyth is only available in testnet network.

### Testnet

The contract address is `terra1hdc8q4ejy82kd9w7wj389dlul9z5zz9a36jflh`

List of available Price Feeds and their ids:

| Symbol         | id (hex)                                                           |
|----------------|--------------------------------------------------------------------|
| Crypto.BTC/USD | 0xf9c0172ba10dfa4d19088d94f5bf61d3b54d5bd7483a322a982e1373ee8ea31b |
| Crypto.ETH/USD | 0xca80ba6dc32e08d06f1aa886011eed1d77c77be9eb761cc10d72b7d0a2fd57a6 |
| Crypto.SOL/USD | 0xfe650f0367d4a7ef9815a593ea15d36593f0643aaaf0149bb04be67ab851decd |
| Crypto.SRM/USD | 0x78ec25615d53d2486db101e829f77615c4408cbbd543088714b9f267da44591a | 
| FX.EUR/USD     | 0xc1b12769f6633798d45adfd62bfc70114839232e2949b01fb3d3f927d2606154 |
| Crypto.BNB/USD | 0xecf553770d9b10965f8fb64771e93f5690a182edc32be4a3236e0caaa6e0581a |
