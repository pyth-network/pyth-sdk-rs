# Pyth SDK Example Program for Solana and the Anchor Library

This example implements the same functionalities as the `sol-contract` example.
The difference is that this example uses the `anchor` library while the `sol-contract` example uses the lower-level solana interface.
Please refer to the README of `sol-contract` for a description of the functionalities.

## Run this program
We assume that you have installed `anchor`, `npm` and `yarn`.

```shell
# Enter the directory and build this example
> cd examples/sol-contract-anchor
> anchor build

# Config the solana CLI by setting its url to devnet
> solana config set --url https://api.devnet.solana.com
# Change the `wallet` field in Anchor.toml to your own wallet
# And then deploy the example contract
> anchor deploy

# Install the client dependencies and invoke this program
> anchor run install
> anchor run invoke
```

You must make sure that the following 3 places have the same program address: (1) Anchor.toml, (2) programs/example-sol-anchor-contract/src/lib.rs and (3) the output of `anchor deploy`.
Otherwise, modify (1) or (2) with the program address shown in (3).
