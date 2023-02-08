# Pyth Network Solana SDK

This crate provides utilities for reading price feeds from the [pyth.network](https://pyth.network/) oracle on the Solana network.
It also includes several [off-chain example programs](examples/).

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

Most users of this SDK only need to access the content of price accounts; the other two account types are implementation details of the oracle.
Applications can obtain the content of these accounts in two different ways:
* On-chain programs should pass these accounts to the instructions that require price feeds.
* Off-chain programs can access these accounts using the Solana RPC client (as in the [eth price example program](examples/eth_price.rs)).

The [pyth.network](https://pyth.network/developers/price-feed-ids#solana-mainnet-beta) website can be used to identify the public keys of each price feed's price account (e.g. Crypto.BTC/USD).

### On-chain

On-chain applications should pass the relevant Pyth Network price account to the Solana instruction that consumes it.
This price account will be represented as an `AccountInfo` in the code for the Solana instruction.
The `load_price_feed_from_account_info` function will construct a `PriceFeed` struct from `AccountInfo`:

```rust
use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};

const STALENESS_THRESHOLD : u64 = 60; // staleness threshold in seconds
let price_account_info: AccountInfo = ...;
let price_feed: PriceFeed = load_price_feed_from_account_info( &price_account_info ).unwrap();
let current_timestamp = Clock::get()?.unix_timestamp;
let current_price: Price = price_feed.get_price_no_older_than(current_timestamp, STALENESS_THRESHOLD).unwrap();
msg!("price: ({} +- {}) x 10^{}", current_price.price, current_price.conf, current_price.expo);
```

The `PriceFeed` object returned by `load_price_feed_from_account_info` contains all currently-available pricing information about the product.
This struct also has some useful functions for manipulating and combining prices; see the [common SDK documentation](../pyth-sdk) for more details.

The function `get_price_no_older_than` takes in an `age` in seconds. If the current on-chain aggregate is older than `current_timestamp - age`, `get_price_no_older_than` will return `None`.

Note that your application should also validate the address of the passed-in price account before using it.
Otherwise, an attacker could pass in a different account and set the price to an arbitrary value.

### Off-chain

Off-chain applications can read the current value of a Pyth Network price account using the Solana RPC client.
This client will return the content of the account as an `Account` struct.
The `load_price_feed_from_account` function will construct a `PriceFeed` struct from `Account`:

```rust
use pyth_sdk_solana::{load_price_feed_from_account, PriceFeed};

const STALENESS_THRESHOLD : u64 = 60; // staleness threshold in seconds
let current_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

let price_key: Pubkey = ...;
let mut price_account: Account = clnt.get_account(&price_key).unwrap();
let price_feed: PriceFeed = load_price_feed_from_account( &price_key, &mut price_account ).unwrap();
let current_price: Price = price_feed.get_price_no_older_than(current_time, STALENESS_THRESHOLD).unwrap();
println!("price: ({} +- {}) x 10^{}", current_price.price, current_price.conf, current_price.expo);
```

## Low-Level Solana Account Structure

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

For more information on the different types of Pyth accounts, see the [account structure documentation](https://docs.pyth.network/how-pyth-works/account-structure).

## Off-chain Example Programs

The example [eth_price](examples/eth_price.rs) program prints the product reference data and current price information for Pyth on pythnet. You can use the same example and replace the url with the relevant Solana cluster urls to get the same information for Solana clusters.
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
