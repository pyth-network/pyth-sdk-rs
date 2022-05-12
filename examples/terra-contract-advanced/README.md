# Pyth SDK Example Contract with Example Mocking

This repository contains an example contract that demonstrates how to use and
mock the Pyth oracle. This also includes a test showing an example of how to
generate mocked Pyth Price data to test against.

The test itself can be found in `contract.rs`, which feeds the contract with
the following price action:

![](./prices.png)

For information on how to build and deploy this contract, refer to the README
of the simple example contract [here](../terra-contract/README.md).
