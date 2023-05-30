# sol-anchor-contract

This is a program developed as a Rust Smart Contract(Solana Blockchain).
It acts as an example for developers who are new to Solana ecosystem to learn on how the program interacts with the Pyth Oracles. 

Instructions of the program:

1. init
2. loanToValue

Please find below instructions on running the Smart Contract on local cluster:

1.

- Open a new terminal
- Run below command which clones two account addresses and their associated data into local cluster from devnet cluster.

solana-test-validator --reset --clone EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw 38xoQ4oeJCBrcVvca2cGk7iV1dAfrmTR1kmhSCJQ8Jto --url devnet

2.

- Open a new terminal.
- Run below command which listens to transaction logs.

solana logs

3.

- Open a new terminal.
- Navigate to the root directory of your application.
- Build and then deploy.
- Run below command

anchor test --skip-local-validator

NB

When testing on local cluster you may encounter error "Pyth price oracle is offline.", please find below one of the approaches of addressing it.
- On file "src\lib.rs", change the value "60" to a higher figure on these lines of code ".get_price_no_older_than(current_timestamp1, 60)"
and ".get_price_no_older_than(current_timestamp2, 60)". This should just be for testing purposes on local cluster.