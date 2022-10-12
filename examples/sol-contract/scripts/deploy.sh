echo "Airdropping..."
solana airdrop 1 --url https://api.devnet.solana.com
echo "Deploying the program..."
solana program deploy --program-id build/example_sol_contract-keypair.json build/example_sol_contract.so
