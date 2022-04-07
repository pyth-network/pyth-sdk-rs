#![cfg(feature = "test-bpf")] // Only runs on bpf, where solana programs run

use pyth_sdk_solana::state::{
    AccountType,
    PriceAccount,
    PriceStatus,
    MAGIC,
    VERSION_2,
};
use solana_program_test::*;
use test_contract::instruction;


mod common;
use common::test_instr_exec_ok;

fn price_account_all_zero() -> PriceAccount {
    PriceAccount {
        magic:      MAGIC,
        ver:        VERSION_2,
        atype:      AccountType::Price as u32,
        ..Default::default()
    }
}


#[tokio::test]
async fn test_price_not_stale() {
    let mut price = price_account_all_zero();
    price.agg.status = PriceStatus::Trading;
    test_instr_exec_ok(instruction::price_status_check(
        &price,
        PriceStatus::Trading,
    ))
    .await;
}


#[tokio::test]
async fn test_price_stale() {
    let mut price = price_account_all_zero();
    price.agg.status = PriceStatus::Trading;
    // Value 100 will cause an overflow because this is bigger than Solana slot in the test suite
    // (its ~1-5). As the check will be 5u - 100u ~= 1e18 > MAX_SLOT_DIFFERENCE. It can only
    // break when Solana slot in the test suite becomes between 100 and 100+MAX_SLOT_DIFFERENCE.
    price.agg.pub_slot = 100;
    test_instr_exec_ok(instruction::price_status_check(
        &price,
        PriceStatus::Unknown,
    ))
    .await;
}
