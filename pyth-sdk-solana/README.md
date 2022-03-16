# Pyth SDK Solana

This crate provides utilities for reading price feeds from the [pyth.network](https://pyth.network/) oracle on the Solana network.
The crate includes a library for reading and using Pyth data feeds in Solana both on-chain (Solana programs) and off-chain (clients interacting with Solana blockchain). It also includes multiple off-chain example programs.

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
pyth-sdk-solana="<version>"
```

See [pyth-sdk-solana on crates.io](https://crates.io/crates/pyth-sdk-solana/) to get the latest version of the library.

## Usage

Pyth Network stores its price feeds in a collection of Solana accounts of various types:
* Price accounts store the current price for a product
* Product accounts store metadata about a product, such as its symbol (e.g., "BTC/USD").
* Mapping accounts store a listing of all Pyth accounts

> :warning: This structure is designed for Pyth Oracle internal program. In most of the use cases only Price account is needed.

For more information on the different types of Pyth accounts, see the [account structure documentation](https://docs.pyth.network/how-pyth-works/account-structure).
The pyth.network website also lists the public keys of the accounts (e.g., [Crypto.BTC/USD accounts](https://pyth.network/markets/#Crypto.BTC/USD)). 

This crate provides utilities for interpreting and manipulating the content of these accounts.
Applications can obtain the content of these accounts in two different ways:
* On-chain programs should pass these accounts to the instructions that require price feeds.
* Off-chain programs can access these accounts using the Solana RPC client (as in the [eth price example program](examples/eth_price.rs)).

In both cases, the content of the account will be provided to the application as a binary blob (`Vec<u8>`).
The examples below assume that the user has already obtained this account data.

### Parse price data
Each price feed (e.g: `Crypto.BTC/USD`) is stored in a Solana price account.
You can find price accounts in the pyth.network website (e.g.: [Crypto.BTC/USD accounts](https://pyth.network/markets/#Crypto.BTC/USD)).  

The `Price` struct contains several useful functions for working with the price.
Some of these functions are described below.
For more detailed information, please see the crate documentation.

#### On-chain

To read the price from a price account on-chain, this library provides a `load_price_from_account_info` method that constructs `Price` struct from AccountInfo:

```rust
use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};

let price_account_info: AccountInfo = ...;
let price: PriceFeed = load_price_feed_from_account_info( &price_account_info ).unwrap();
```

#### Off-chain
To read the price from a price account off-chain in clients, this library provides a `load_price_from_account` method that constructs `Price` struct from Account:

```rust
use pyth_sdk_solana::{load_price_feed_from_account, PriceFeed};

let price_key: Pubkey = ...;
let mut price_account: Account = ...;
let price: PriceFeed = load_price_feed_from_account( &price_key, &mut price_account ).unwrap();
```

### Get the current price

Read the current price from a `PriceFeed`:

```rust
let current_price: Price = price_feed.get_current_price().unwrap();
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

### Solana Account Structure

> :warning: The Solana account structure is an internal API that is subject to change. Prefer to use `load_price_feed_*` when possible.

This library also provides several `load_*` methods that allow users to translate the binary data in each account into an appropriate struct:

```rust
use pyth_sdk_solana::state::*;

// replace with account data, either passed to on-chain program or from RPC node 
let price_account_data: Vec<u8> = ...;
let price_account: &PriceAccount = load_price_account( &price_account_data ).unwrap();

let product_account_data: Vec<u8> = ...;
let product_account: &ProductAccount = load_product_account( &product_account_data ).unwrap();

let mapping_account_data: Vec<u8> = ...;
let mapping_account: &MappingAccount = load_mapping_account( &mapping_account_data ).unwrap();
```


### Off-chain example program

The example [eth_price](examples/eth_price.rs) program prints the product reference data and current price information for Pyth on Solana devnet.
Run the following commands to try this example program:

```
cargo build --examples
cargo run --example eth_price
```

The output of this command is price of ETH/USD over time, such as:

```
.....ETH/USD.....
status .......... Trading
num_publishers .. 19
price ........... 291958500000 x 10^-8
conf ............ 163920000 x 10^-8
ema_price ....... 291343470000 x 10^-8
ema_conf ........ 98874533 x 10^-8
```

For [an example](examples/get_accounts.rs) of using Solana Account structure please run:
```
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
    ema_price .... 7426390900
    ema_conf ..... 2259870
```

## Development

This library can be built for either your native platform or in BPF (used by Solana programs). 
Use `cargo build` / `cargo test` to build and test natively.
Use `cargo build-bpf` / `cargo test-bpf` to build in BPF for Solana; these commands require you to have installed the [Solana CLI tools](https://docs.solana.com/cli/install-solana-cli-tools). 
