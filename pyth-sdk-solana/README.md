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

### Price Feeds

Pyth Network stores the data for it's price feeds in Solana accounts, called "price accounts".

Applications can obtain the content of these accounts in two different ways:
* On-chain programs should pass these accounts to the instructions that require price feeds.
* Off-chain programs can access these accounts using the Solana RPC client (as in the [eth price example program](examples/eth_price.rs)).

To use the SDK, you will need to find the price feed account for the symbol you wish to consume. The [Pyth Network documentation](https://docs.pyth.network/consume-data/solana#price-feeds) explains how to do this. The public key of this account corresponds to the ID of the price feed.

### On-chain

On-chain applications should pass the relevant Pyth Network price account to the Solana instruction that consumes it.
This price account will be represented as an `AccountInfo` in the code for the Solana instruction.
The `load_price_feed_from_account_info` function will construct a `PriceFeed` struct from `AccountInfo`:

```rust
use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};

let price_account_info: AccountInfo = ...;
let price_feed: PriceFeed = load_price_feed_from_account_info( &price_account_info ).unwrap();
let current_price: Price = price_feed.get_current_price().unwrap();
println!("price: ({} +- {}) x 10^{}", current_price.price, current_price.conf, current_price.expo);
```

The `PriceFeed` object returned by `load_price_feed_from_account_info` contains all currently-available pricing information about the product.
This struct also has some useful functions for manipulating and combining prices; see the [common SDK documentation](../pyth-sdk) for more details.

Note that your application should also validate the address of the passed-in price account before using it.
Otherwise, an attacker could pass in a different account and set the price to an arbitrary value.

### Off-chain

Off-chain applications can read the current value of a Pyth Network price account using the Solana RPC client.
This client will return the content of the account as an `Account` struct.
The `load_price_feed_from_account` function will construct a `PriceFeed` struct from `Account`:

```rust
use pyth_sdk_solana::{load_price_feed_from_account, PriceFeed};

let price_key: Pubkey = ...;
let mut price_account: Account = ...;
let price_feed: PriceFeed = load_price_feed_from_account( &price_key, &mut price_account ).unwrap();
let current_price: Price = price_feed.get_current_price().unwrap();
println!("price: ({} +- {}) x 10^{}", current_price.price, current_price.conf, current_price.expo);
```

## Off-chain Example Program

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

## Development

This library can be built for either your native platform or in BPF (used by Solana programs).
Use `cargo build` / `cargo test` to build and test natively.
Use `cargo build-bpf` / `cargo test-bpf` to build in BPF for Solana; these commands require you to have installed the [Solana CLI tools](https://docs.solana.com/cli/install-solana-cli-tools).
