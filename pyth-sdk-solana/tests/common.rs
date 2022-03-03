use {
    pyth_client::id,
    pyth_client::processor::process_instruction,
    solana_program::instruction::Instruction,
    solana_program_test::*,
    solana_sdk::{signature::Signer, transaction::Transaction},
};

// Panics if running instruction fails
pub async fn test_instr_exec_ok(instr: Instruction) {
    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "pyth_client",
        id(),
        processor!(process_instruction),
    )
        .start()
        .await;
    let mut transaction = Transaction::new_with_payer(
        &[instr],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap()
}
