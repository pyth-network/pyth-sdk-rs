# Pyth Network SDK

The Pyth Network Rust SDK provides utilities for reading price feeds from the [pyth.network](https://pyth.network/) oracle in on- and off-chain applications.

Key features of this SDK include:

* Get the current price of over [50 products](https://pyth.network/markets/), including cryptocurrencies,
  US equities, forex and more.
* Combine listed products to create new price feeds, e.g., for baskets of tokens or non-USD quote currencies.
* Consume prices in Solana programs, Terra smart contracts, or off-chain applications.

Please see the [pyth.network documentation](https://docs.pyth.network/) for more information about pyth.network.

## Usage

This repository is divided into several crates focused on specific use cases:

1. [Pyth SDK](pyth-sdk) provides common data types and interfaces for that are shared across different blockchains.
2. [Pyth SDK Solana](pyth-sdk-solana) provides an interface for reading Pyth price feeds in Solana programs.
   This crate may also be used in off-chain applications that read prices from the Solana blockchain.
3. [Pyth SDK Terra](pyth-sdk-terra) provides an interface for reading Pyth price feeds in on-chain Terra contracts.

Please see the documentation for the relevant crate to get started using Pyth Network.

## Development

All crates in this repository can be built for either your native platform or blockchain-specific platforms.
Use `cargo build` / `cargo test` to build and test natively.

### Schema Files

JSON Schema files are provided to allow others to work with the various Pyth structures in languages other than Rust. These are also used within Pyth’s own repositories, for example within the pyth-sdk-js repo. Every time these structures change, new Schema’s must be generated and committed via the cargo run --example schema command.

There is currently a CI check which ensures the schema files remain up-to-date.

### Creating a Release

To release a new version of any of these crates, perform the following steps within the crate being released:

1. Increment the version number in `Cargo.toml`.
   You may use a version number with a `-beta.x` suffix such as `0.0.1-beta.0` to create opt-in test versions.
2. Merge your change into `main` on github.
3. Create and publish a new github release with the structure of `package:vx.y.z` such as `pyth-sdk-solana:v1.3.2`. It will trigger a github action
   and publishes the released package in [crates.io](https://crates.io)

### pre-commit hooks
pre-commit is a tool that checks and fixes simple issues (formatting, ...) before each commit. You can install it by following [their website](https://pre-commit.com/). In order to enable checks for this repo run `pre-commit install` from command-line in the root of this repo.

The checks are also performed in the CI to ensure the code follows consistent formatting.
