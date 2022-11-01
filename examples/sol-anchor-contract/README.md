# Pyth SDK Example Program for Solana and the Anchor Library

This example implements the same functionalities as the `sol-contract` example.
The difference is that this example uses the `anchor` library while the `sol-contract` example uses the lower-level solana interface.
Please refer to the README of `sol-contract` for a description of the functionalities.

## Run this program
We assume that you have installed `anchor`, `npm` and `yarn`.

```shell
# Generate the program key
> solana-keygen new -o program_address.json

# Use the pubkey generated to replace the following two places
# "example_sol_anchor_contract" in Anchor.toml
# "declare_id!()" in programs/example-sol-anchor-contract/src/lib.rs

# Enter the directory and build this example
> cd examples/sol-contract-anchor
> anchor build

# Change the `wallet` field in Anchor.toml to your own wallet
# And then deploy the example contract; An error may occur if
# your wallet does not have enough funds
> anchor deploy --program-keypair program_address.json --program-name example-sol-anchor-contract

# Install the client dependencies and invoke this program
> anchor run install
> anchor run invoke
```
