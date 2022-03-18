# Pyth SDK Example contract for Terra

This is an example contract that demonstrates reading the Pyth price from the Pyth on-chain contract. It is created using
[cw-template](https://github.com/InterWasm/cw-template) which is a standard template for developing Terra contracts.

## Contract API

### Instantiation
It will take Pyth contract address and a price feed id as it's configuration.

### Execution
This contract won't execute any logic.

### Query
It provides a `fetch_price` interface which will return the price of the configured price feed if the price is available.

## Using project

Check out [Developing](./Developing.md) to learn more on how to run tests and develop code.

Also check out [`tools` directory README](./tools/README.md) to learn how to deploy the contract and query it on-chain.