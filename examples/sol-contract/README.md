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

## Where and how the Pyth SDK is used?
Pyth SDK is used in the `Loan2Value` instruction in `src/processor.rs`.
For the loan, the code first reads the unit price from the Pyth oracle.
```rust
let feed1 = load_price_feed_from_account_info(pyth_loan_account)?;
let result1 = feed1.get_current_price().ok_or(ProgramError::Custom(3))?;
```

And then calculate the loan value given the quantity of the loan.
```rust
let loan_value = result1
    .price
    .checked_mul(loan_info.loan_qty)
    .ok_or(ProgramError::Custom(4))?;
let loan_conf = (result1.conf as f64)       // confidence
    * (10 as f64).powf(result1.expo as f64) // * 10 ^ exponent
    * (loan_info.loan_qty as f64);          // * quantity
let loan_value_max = loan_value as f64 + loan_conf;
```

This code says that, with high confidence, the maximum value of the loan does not exceed `loan_value_max`.
In a similar way, the code then calculates the minimum value of the collateral and compare the two.

## Run this program
We assume that you have installed `cargo`, `solana`, `npm` and `node`.

```shell
# Enter the root directory of this example
> cd examples/sol-contract
# Build the example contract
> scripts/build.sh
# Deploy the example contract
> scripts/deploy.sh
# Invoke the example contract
> scripts/invoke.ts
```
