//! Program entrypoint
//! Every solana program has an entry point function with 3 parameters:
//! the program ID, the accounts being touched by this program,
//! and an arbitrary byte array as the input data for execution.

use solana_program::entrypoint;
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    crate::processor::process_instruction(
        program_id, accounts, instruction_data
    )
}
