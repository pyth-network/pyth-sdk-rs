# Pyth Network CosmosWasm SDK

This crate provides utilities for reading price feeds from the [pyth.network](https://pyth.network/) oracle on the CosmWasm ecosystem.
It also includes an [example contract](../examples/terra-contract/) demonstrating how to read price feeds from on-chain CosmWasm applications.

## Installation

Add this crate to the dependencies section of your CosmWasm contract's `Cargo.toml` file:

```
[dependencies]
pyth-sdk-cw = { version = "<current version>" }
```

See [pyth-sdk-cw on crates.io](https://crates.io/crates/pyth-sdk-cw) to get the most recent version.

## Usage

Simply call the `query_price_feed` function in your CosmWasm contract with a price feed id:

```rust
// Pyth network testnet contract address
pyth_contract_addr = deps.api.addr_validate("terra1wzs3rgzgjdde3kg7k3aaz6qx7sc5dcwxqe9fuc")?;
// Price feed id for BTC/USD on testnet
price_feed_id = PriceIdentifier::from_hex("f9c0172ba10dfa4d19088d94f5bf61d3b54d5bd7483a322a982e1373ee8ea31b");

let price_feed: PriceFeed = query_price_feed(deps.querier, pyth_contract_addr, price_feed_id)?.price_feed;
let current_price: Price = price_feed.get_current_price().ok_or_else(|| StdError::not_found("price is not currently available"))?;
println!("current BTC/USD price: ({} +- {}) x 10^{}", current_price.price, current_price.conf, current_price.expo);
```

`query_price_feed` makes a query to the Pyth Network CosmWasm contract
This query requires a price feed id that indicates the product whose price should be returned.
Each product listed on Pyth Network (e.g., BTC/USD) has its own price feed id; see the [Contracts and Price Feeds](#contracts-and-price-feeds) section below for the possible products and their price feed ids.
The result of the query is a `PriceFeed` struct which contains the current price of the product along with additional metadata.
This struct also has some useful functions for manipulating and combining prices; see the [common SDK documentation](../pyth-sdk) for more details.

## Off-Chain Queries

You can use the provided schemas in the `schema` directory to directly query the CosmWasm contract from off-chain applications.
A typical query requires to pass the price feed id as a hex string. it will look like:

```
{
    "price_feed": {
        "id": "f9c0172ba10dfa4d19088d94f5bf61d3b54d5bd7483a322a982e1373ee8ea31b"
    }
}
```

By going to the contract address in [Terra Finder](https://finder.terra.money/) you can try and make a query for a price feed and see the result.

## Contracts and Price Feeds

Pyth is currently only available in Terra Classic testnet.

### Terra Classic testnet

The contract address is [`terra1wzs3rgzgjdde3kg7k3aaz6qx7sc5dcwxqe9fuc`](https://finder.terra.money/testnet/address/terra1wzs3rgzgjdde3kg7k3aaz6qx7sc5dcwxqe9fuc).

You can find a list of available price feeds [here](https://pyth.network/developers/price-feeds/#terra-testnet)
