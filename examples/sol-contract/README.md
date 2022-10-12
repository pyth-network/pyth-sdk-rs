# Pyth SDK Example Program for Solana

This is an example demonstrating how to read prices from Pyth on Solana.

The program has two instructions: `Init` and `Loan2Value`.
`Init` can *only* be invoked by the program admin and it will initialize some loan information.
`Loan2Value` can be invoked by anyone and it uses the current Pyth price to compare the value of the loan and the value of the collateral.
This is an important functionality in many lending protocols.

The key program logic is in 3 files.
The loan information structure is defined in `src/state.rs`, which also contains the serialization and deserialization code.
The two instructions are implemented in `src/processor.rs`.
An example invocation of these instructions on the Solana devnet can be found in `scripts/invoke.ts`.

## Where and how is the Pyth SDK used?
Pyth SDK is used in the `Loan2Value` instruction in `src/processor.rs`.
For the loan, the code first reads the unit price from the Pyth oracle.
```rust
let feed1 = load_price_feed_from_account_info(pyth_loan_account)?;
let result1 = feed1.get_current_price().ok_or(ProgramError::Custom(3))?;
```

And then calculate the loan value given the quantity of the loan.
```rust
let loan_max_price = result1
    .price
    .checked_add(result1.conf as i64)
    .ok_or(ProgramError::Custom(4))?;
let loan_max_value = loan_max_price
    .checked_mul(loan_qty)
    .ok_or(ProgramError::Custom(4))?;
```

This code says that, with high confidence, the maximum value of the loan does not exceed `loan_max_value * 10^(result1.expo)` at the time of the query.
In a similar way, the code then calculates the minimum value of the collateral and compare the two.

More on Pyth best practice and price confidence interval can be found [here](https://docs.pyth.network/consume-data/best-practices).

## Run this program
We assume that you have installed `cargo`, `solana`, `npm` and `node`.

```shell
# Enter the root directory of this example
> cd examples/sol-contract
# Build the example contract
> scripts/build.sh
# Config solana CLI and set the url as devnet
> solana config set --url https://api.devnet.solana.com
# Deploy the example contract
> scripts/deploy.sh
# Invoke the example contract
> scripts/invoke.ts
```
