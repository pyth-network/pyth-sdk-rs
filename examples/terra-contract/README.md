# Pyth SDK Example contract for Terra

This is an example contract that demonstrates using Pyth price on-chain. 

## Intro

This contracts acts as a proxy to a Pyth Price in Terra.

### Instantiation
It will take Pyth contract address and a price feed is as it's configuration.

### Execution
This contract won't execute any logic.

### Query
It provides a `fetch_price` interface which will return the price of the configured price feed if the price is available.

## Using project

Check out [Developing](./Developing.md) to learn more on how to run tests and develop code.

Also check out [`tools` directory README](./tools/README.md) to learn how to deploy the contract and query it on-chain.