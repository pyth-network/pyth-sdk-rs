# Pyth SDK Solana Test Contract
This contract is used to test pyth-sdk-solana onchain.

## Development
Use `cargo build-bpf` / `cargo test-bpf` to build in BPF for Solana; these commands require you to have installed the [Solana CLI tools](https://docs.solana.com/cli/install-solana-cli-tools).

The BPF tests will also run an instruction count program that logs the resource consumption
of various library functions.
This program can also be run on its own using `cargo test-bpf --test instruction_count`.
