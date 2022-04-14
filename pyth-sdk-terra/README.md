# Pyth Network Terra SDK

This crate provides utilities for reading price feeds from the [pyth.network](https://pyth.network/) oracle on the Terra network.
It also includes an [example contract](../examples/terra-contract/) demonstrating how to read price feeds from on-chain Terra applications.

## Installation

Add this crate to the dependencies section of your Terra contract's `Cargo.toml` file:

```
[dependencies]
pyth-sdk-terra = { version = "<current version>" }
```

See [pyth-sdk-terra on crates.io](https://crates.io/crates/pyth-sdk-terra) to get the most recent version.

## Usage

Simply call the `query_price_feed` function in your Terra contract with a price feed id:

```rust
// Pyth network testnet contract address
pyth_contract_addr: string = "terra1hdc8q4ejy82kd9w7wj389dlul9z5zz9a36jflh";
// Price feed id for BTC/USD on testnet
price_feed_id = PriceIdentifier::from_hex("f9c0172ba10dfa4d19088d94f5bf61d3b54d5bd7483a322a982e1373ee8ea31b");

let price_feed: PriceFeed = query_price_feed(deps.querier, pyth_contract_addr, price_feed_id)?.price_feed;
let current_price: Price = price_feed.get_current_price().ok_or_else(|| StdError::not_found("price is not currently available"))?;
println!("current BTC/USD price: ({} +- {}) x 10^{}", current_price.price, current_price.conf, current_price.expo);
```

`query_price_feed` makes a query to the Pyth Network Terra contract
This query requires a price feed id that indicates the product whose price should be returned.
Each product listed on Pyth Network (e.g., BTC/USD) has its own price feed id; see the [Contracts and Price Feeds](#contracts-and-price-feeds) section below for the possible products and their price feed ids.
The result of the query is a `PriceFeed` struct which contains the current price of the product along with additional metadata.
This struct also has some useful functions for manipulating and combining prices; see the [common SDK documentation](../pyth-sdk) for more details.

## Off-Chain Queries

You can use the provided schemas in the `schema` directory to directly query the terra contract from off-chain applications.
A typical query will look like:

```
{
    "price_feed": {
        "id": "f9c0172ba10dfa4d19088d94f5bf61d3b54d5bd7483a322a982e1373ee8ea31b" // id of the price feed (in hex format)
    }
}
```

By going to the contract address in [Terra Finder](https://finder.terra.money/) you can try and make a query for a price feed and see the result.

## Contracts and Price Feeds

Pyth is currently only available in Terra testnet.

### Testnet

The contract address is [`terra1hdc8q4ejy82kd9w7wj389dlul9z5zz9a36jflh`](https://finder.terra.money/testnet/address/terra1wzs3rgzgjdde3kg7k3aaz6qx7sc5dcwxqe9fuc).

List of available Price Feeds and their ids:

| Symbol          | id (hex)                                                             |
|-----------------|----------------------------------------------------------------------|
| Crypto.BTC/USD  | `0xf9c0172ba10dfa4d19088d94f5bf61d3b54d5bd7483a322a982e1373ee8ea31b` |
| Crypto.ETH/USD  | `0xca80ba6dc32e08d06f1aa886011eed1d77c77be9eb761cc10d72b7d0a2fd57a6` |
| Crypto.LUNA/USD | `0x6de025a4cf28124f8ea6cb8085f860096dbc36d9c40002e221fc449337e065b2` |
| Crypto.UST/USD  | `0x026d1f1cf9f1c0ee92eb55696d3bd2393075b611c4f468ae5b967175edc4c25c` |
| Crypto.ALGO/USD | `0x08f781a893bc9340140c5f89c8a96f438bcfae4d1474cc0f688e3a52892c7318` |

Testnet price feeds update once per minute.

#### Notes
- :warning: `num_publishers` and `max_num_publishers` in `PriceFeed` are currently unavailable and set to 0.
