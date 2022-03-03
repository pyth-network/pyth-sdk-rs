use pyth_sdk_solana::id;
use pyth_sdk_solana::processor::process_instruction;
use solana_program::instruction::Instruction;
use solana_program_test::*;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;

// Panics if running instruction fails
pub async fn test_instr_exec_ok(instr: Instruction) {
    let (mut banks_client, payer, recent_blockhash) =
        ProgramTest::new("pyth_sdk_solana", id(), processor!(process_instruction))
            .start()
            .await;
    let mut transaction = Transaction::new_with_payer(&[instr], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap()
}
