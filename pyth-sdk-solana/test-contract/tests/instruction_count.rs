use solana_program_test::*;
use test_contract::instruction;

use pyth_sdk_solana::Price;

mod common;
use common::test_instr_exec_ok;

fn pc(price: i64, conf: u64, expo: i32) -> Price {
    Price {
        price,
        conf,
        expo,
        publish_time: 0,
    }
}

#[tokio::test]
async fn test_noop() {
    test_instr_exec_ok(instruction::noop()).await;
}

#[tokio::test]
async fn test_scale_to_exponent_down() {
    test_instr_exec_ok(instruction::scale_to_exponent(pc(1, u64::MAX, -1000), 1000)).await
}

#[tokio::test]
async fn test_scale_to_exponent_up() {
    test_instr_exec_ok(instruction::scale_to_exponent(pc(1, u64::MAX, 1000), -1000)).await
}

#[tokio::test]
async fn test_scale_to_exponent_best_case() {
    test_instr_exec_ok(instruction::scale_to_exponent(pc(1, u64::MAX, 10), 10)).await
}

#[tokio::test]
async fn test_normalize_max_conf() {
    test_instr_exec_ok(instruction::normalize(pc(1, u64::MAX, 0))).await
}

#[tokio::test]
async fn test_normalize_max_price() {
    test_instr_exec_ok(instruction::normalize(pc(i64::MAX, 1, 0))).await
}

#[tokio::test]
async fn test_normalize_min_price() {
    test_instr_exec_ok(instruction::normalize(pc(i64::MIN, 1, 0))).await
}

#[tokio::test]
async fn test_normalize_best_case() {
    test_instr_exec_ok(instruction::normalize(pc(1, 1, 0))).await
}

#[tokio::test]
async fn test_div_max_price() {
    test_instr_exec_ok(instruction::divide(pc(i64::MAX, 1, 0), pc(1, 1, 0))).await;
}

#[tokio::test]
async fn test_div_max_price_2() {
    test_instr_exec_ok(instruction::divide(pc(i64::MAX, 1, 0), pc(i64::MAX, 1, 0))).await;
}

#[tokio::test]
async fn test_mul_max_price() {
    test_instr_exec_ok(instruction::multiply(pc(i64::MAX, 1, 2), pc(123, 1, 2))).await;
}

#[tokio::test]
async fn test_mul_max_price_2() {
    test_instr_exec_ok(instruction::multiply(
        pc(i64::MAX, 1, 2),
        pc(i64::MAX, 1, 2),
    ))
    .await;
}
