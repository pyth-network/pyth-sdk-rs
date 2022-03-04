# Pyth SDK

This repo contains multiple crates for using Pyth Oracle.
1. Pyth SDK: This crate contains general Pyth structures and interfaces which are consistent across different blockchains.
2. Pyth SDK Solana: This crate contains methods for reading and parsing Pyth structures from Pyth Solana accounts.

## Development

These crates can be built for either your native platform or other platforms for specific blockchains.
- Use `cargo build` / `cargo test` to build and test natively.

### Releases

To release a new version of this package, perform the following steps:

1. Increment the version number in `Cargo.toml`.
   You may use a version number with a `-beta.x` suffix such as `0.0.1-beta.0` to create opt-in test versions.
2. Merge your change into `main` on github.
3. Create and publish a new github release.
   Publishing the release will trigger a github action that will automatically publish the [pyth-sdk](https://crates.io/crates/pyth-sdk) or [pyth-sdk-solana](https://crates.io/crates/pyth-sdk-solana) rust crate to `crates.io`.
