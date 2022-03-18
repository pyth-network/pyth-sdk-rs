# Pyth SDK Example contract for Terra

This is an example contract that demonstrates reading the Pyth price from the Pyth on-chain contract. It is created using
[cw-template](https://github.com/InterWasm/cw-template) which is a standard template for developing Terra contracts.

## Development

Visit [Developing](./Developing.md) to learn more on how to compile and develop the contract.

## Deploy and Query
The javascript package in `tools` directory contains scripts for querying and deploying the example contract.

If this is the first time running the code, run the below command to install required packages in the `tools` directory: 

```
npm install
```

### Testnet Demo
In order to query the contract you can call:

```sh
npm run query -- --network testnet --contract terra1fm4ssxq39m355pdv2wzxggf5uxs2ase4vga9qs
```

If successful the output should look like:
```
{
  current_price: { price: 8704350000, conf: 3150000, expo: -8 },
  ema_price: { price: 8665158600, conf: 2965370, expo: -8 }
}
```

If the price is not available you will get:
```
rpc error: code = Unknown desc = Generic error: Current price is not available: contract query failed
```

`terra1fm4ssxq39m355pdv2wzxggf5uxs2ase4vga9qs` is a live deployment of the example contract in testnet network. This contract
is configured to return price of `Crypto.LUNA/USD` if it is available. If you have deployed your contract you can replace the 
address with your contract address.
### Deployment

Deploying a contract in terra consists of two steps:
1. Uploading the code. This step will give you a code id.
2. Optionally create a new contract or migrate an existing one:
    1. Creating a new contract which has an address with a code id as its program.
    2. Migrating an existing contract code id to the new code id.

This script can do both steps at the same time. Read below for the details.

#### Uploading the code

First build the contracts as mentioned in [Developing](../Developing.md).

This command will builds and saves all the contracts in the `artifact` directory.

Then, for example, to deploy `example_terra_contract.wasm`, run in the `tools` directory:

``` sh
npm run deploy -- --network testnet --artifact ../artifacts/example_terra_contract.wasm --mnemonic "..."
```

which will print something along the lines of:

``` sh
Storing WASM: ../artifacts/example_terra_contract.wasm (367689 bytes)
Deploy fee:  88446uluna
Code ID:  2435
```

If you do not pass any additional arguments to the script it will only upload the code and returns the code id. If you want to create a 
new contract or upgrade an existing contract you should pass more arguments that are described below.

#### Instantiating a new contract
If you want instantiate a new contract after your deployment pass `--instantiate` argument to the above command.
It will upload the code and with the resulting code id instantiates a new example contract:

``` sh
npm run deploy -- --network testnet --artifact ../artifacts/example_terra_contract.wasm --mnemonic "..." --instantiate
```

If successful, the output should look like:
```
Storing WASM: ../artifacts/example_terra_contract.wasm (183749 bytes)
Deploy fee:  44682uluna
Code ID:  53199
Instantiating a contract
Sleeping for 10 seconds for store transaction to finalize.
Instantiated Pyth Example at terra123456789yelw23uh22nadqlyjvtl7s5527er97 (0x0000000000000000000000001234567896267ee5479752a7d683e49317ff4294)
Deployed pyth example contract at terra123456789yelw23uh22nadqlyjvtl7s5527er97
```

This scripts currently set the example contract price to `Crypto.LUNA/USD` but you can change it within `deploy.js`.

#### Migrating an existing contract
If you want to upgrade an existing contract pass `--migrate --contract terra123456xyzqwe..` arguments to the above command.
It will upload the code and with the resulting code id migrates the existing contract to the new one:

``` sh
npm run deploy -- --network testnet --artifact ../artifacts/example_terra_contract.wasm --mnemonic "..." --migrate --contract "terra123..."
```

If successful, the output should look like:
```
Storing WASM: ../artifacts/example_terra_contract.wasm (183749 bytes)
Deploy fee:  44682uluna
Code ID:  53227
Sleeping for 10 seconds for store transaction to finalize.
Migrating contract terra1rhjej5gkyelw23uh22nadqlyjvtl7s5527er97 to 53227
Contract terra1rhjej5gkyelw23uh22nadqlyjvtl7s5527er97 code_id successfully updated to 53227
```

#### Notes

You might encounter gateway timeout or account sequence mismatch in errors. In is good to double check with terra finder as sometimes
transactions succeed despite being timed out.

If that happens in the middle of an instantiation or migration. You can avoid re-uploading the code and use the resulting Code Id 
by passing `--code-id <codeId>` instead of `--artifact` and it will only do the instantiation/migration part.

An example is:

``` sh
npm run deploy -- --network testnet --code-id 50123 --mnemonic "..." --migrate --contract "terra123..."
```
