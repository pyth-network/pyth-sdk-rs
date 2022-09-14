# Pyth SDK Example Contract for Solana

This repository contains a simple example demonstrating how to read the Pyth price from the Pyth contract on Solana.

The key functionality of this contract is in the `Loan2Value` function in `src/processor.rs`. 
This function compares the value of some loan and some collateral, which is important in many lending protocols.
An example invocation of this contract on the Solana devnet can be find in `scripts/invoke.ts`.

We assume that you have installed `cargo`, `solana`, `solana-keygen`, `npm` and `node`.

## Usage

```
# To build the example contract
> scripts/build.sh
# To deploy the example contract
> scripts/deploy.sh
# To invoke the example contract
> scripts/invoke.ts
```
