use pyth_sdk_solana::state::{
    AccountType,
    PriceAccount,
    PriceStatus,
    MAGIC,
    VERSION_2,
};
use pyth_sdk_solana::VALID_SLOT_PERIOD;
use solana_program_test::*;
use test_contract::instruction;


mod common;
use common::test_instr_exec_ok;

fn price_account_all_zero() -> PriceAccount {
    PriceAccount {
        magic: MAGIC,
        ver: VERSION_2,
        atype: AccountType::Price as u32,
        ..Default::default()
    }
}


#[tokio::test]
async fn test_price_not_stale() {
    let mut price = price_account_all_zero();
    price.agg.pub_slot = 1000 - 10;
    price.agg.status = PriceStatus::Trading;
    test_instr_exec_ok(instruction::price_status_check(
        &price,
        PriceStatus::Trading,
    ))
    .await;
}


#[tokio::test]
async fn test_price_not_stale_future() {
    let mut price = price_account_all_zero();
    price.agg.pub_slot = 1000 + 10;
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
    price.agg.pub_slot = 1000 - VALID_SLOT_PERIOD - 1;

    #[cfg(feature = "test-bpf")] // Only in BPF the clock check is performed
    let expected_status = PriceStatus::Unknown;

    #[cfg(not(feature = "test-bpf"))]
    let expected_status = PriceStatus::Trading;

    test_instr_exec_ok(instruction::price_status_check(&price, expected_status)).await;
}
