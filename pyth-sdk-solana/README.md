# Pyth Client

This crate provides utilities for reading price feeds from the [pyth.network](https://pyth.network/) oracle on the Solana network.
The crate includes a library for on-chain programs and an off-chain example program.

Key features of this library include:

* Get the current price of over [50 products](https://pyth.network/markets/), including cryptocurrencies,
  US equities, forex and more.
* Combine listed products to create new price feeds, e.g., for baskets of tokens or non-USD quote currencies.
* Consume prices in on-chain Solana programs or off-chain applications.

Please see the [pyth.network documentation](https://docs.pyth.network/) for more information about pyth.network.

## Installation

Add a dependency to your Cargo.toml:

```toml
[dependencies]
pyth-client="<version>"
```

If you want to use this library in your on-chain program you should use `no-entrypoint` feature to prevent conflict between your program and this library's program.

```toml
[dependencies]
pyth-client = {version = "<version>", features = ["no-entrypoint"]}
```

See [pyth-client on crates.io](https://crates.io/crates/pyth-client/) to get the latest version of the library.

## Usage

Pyth Network stores its price feeds in a collection of Solana accounts.
This crate provides utilities for interpreting and manipulating the content of these accounts.
Applications can obtain the content of these accounts in two different ways:
* On-chain programs should pass these accounts to the instructions that require price feeds.
* Off-chain programs can access these accounts using the Solana RPC client (as in the [example program](examples/get_accounts.rs)).

In both cases, the content of the account will be provided to the application as a binary blob (`Vec<u8>`).
The examples below assume that the user has already obtained this account data.

### Parse account data

Pyth Network has several different types of accounts:
* Price accounts store the current price for a product
* Product accounts store metadata about a product, such as its symbol (e.g., "BTC/USD").
* Mapping accounts store a listing of all Pyth accounts

For more information on the different types of Pyth accounts, see the [account structure documentation](https://docs.pyth.network/how-pyth-works/account-structure).
The pyth.network website also lists the public keys of the accounts (e.g., [BTC/USD accounts](https://pyth.network/markets/#BTC/USD)).  

This library provides several `load_*` methods that translate the binary data in each account into an appropriate struct: 

```rust
// replace with account data, either passed to on-chain program or from RPC node 
let price_account_data: Vec<u8> = ...;
let price_account: Price = load_price( &price_account_data ).unwrap();

let product_account_data: Vec<u8> = ...;
let product_account: Product = load_product( &product_account_data ).unwrap();

let mapping_account_data: Vec<u8> = ...;
let mapping_account: Mapping = load_mapping( &mapping_account_data ).unwrap();
```

### Get the current price

Read the current price from a `Price` account: 

```rust
let price: PriceConf = price_account.get_current_price().unwrap();
println!("price: ({} +- {}) x 10^{}", price.price, price.conf, price.expo);
```

The price is returned along with a confidence interval that represents the degree of uncertainty in the price.
Both values are represented as fixed-point numbers, `a * 10^e`. 
The method will return `None` if the price is not currently available.

The status of the price feed determines if the price is available. You can get the current status using:

```rust
let price_status: PriceStatus = price_account.get_current_price_status();
```

### Non-USD prices 

Most assets in Pyth are priced in USD.
Applications can combine two USD prices to price an asset in a different quote currency:

```rust
let btc_usd: Price = ...;
let eth_usd: Price = ...;
// -8 is the desired exponent for the result 
let btc_eth: PriceConf = btc_usd.get_price_in_quote(&eth_usd, -8);
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
let basket_price: PriceConf = Price::price_basket(&[
    (btc_usd, 10, -2),
    (eth_usd, 5, -2)
  ], -8);
println!("0.1 BTC and 0.05 ETH are worth: ({} +- {}) x 10^{} USD",
         basket_price.price, basket_price.conf, basket_price.expo);
```

This function additionally propagates any uncertainty in the price into uncertainty in the value of the basket.

### Off-chain example program

The example program prints the product reference data and current price information for Pyth on Solana devnet.
Run the following commands to try this example program:

```
cargo build --examples
cargo run --example get_accounts
```

The output of this command is a listing of Pyth's accounts, such as:

```
product_account .. 6MEwdxe4g1NeAF9u6KDG14anJpFsVEa2cvr5H6iriFZ8
  symbol.......... SRM/USD
  asset_type...... Crypto
  quote_currency.. USD
  description..... SRM/USD
  generic_symbol.. SRMUSD
  base............ SRM
  price_account .. 992moaMQKs32GKZ9dxi8keyM2bUmbrwBZpK4p2K6X5Vs
    price ........ 7398000000
    conf ......... 3200000
    price_type ... price
    exponent ..... -9
    status ....... trading
    corp_act ..... nocorpact
    num_qt ....... 1
    valid_slot ... 91340924
    publish_slot . 91340925
    twap ......... 7426390900
    twac ......... 2259870
```

## Development

This library can be built for either your native platform or in BPF (used by Solana programs). 
Use `cargo build` / `cargo test` to build and test natively.
Use `cargo build-bpf` / `cargo test-bpf` to build in BPF for Solana; these commands require you to have installed the [Solana CLI tools](https://docs.solana.com/cli/install-solana-cli-tools). 

The BPF tests will also run an instruction count program that logs the resource consumption
of various library functions.
This program can also be run on its own using `cargo test-bpf --test instruction_count`.

### Releases

To release a new version of this package, perform the following steps:

1. Increment the version number in `Cargo.toml`.
   You may use a version number with a `-beta.x` suffix such as `0.0.1-beta.0` to create opt-in test versions.
2. Merge your change into `main` on github.
3. Create and publish a new github release.
   The name of the release should be the version number, and the tag should be the version number prefixed with `v`.
   Publishing the release will trigger a github action that will automatically publish the [pyth-client](https://crates.io/crates/pyth-client) rust crate to `crates.io`.
