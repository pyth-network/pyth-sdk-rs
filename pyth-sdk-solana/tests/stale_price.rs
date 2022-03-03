#![cfg(feature = "test-bpf")] // Only runs on bpf, where solana programs run

use {
    pyth_client::{MAGIC, VERSION_2, instruction, PriceType, Price, AccountType, AccKey, Ema, PriceComp, PriceInfo, CorpAction, PriceStatus},
    solana_program_test::*,
};


mod common;
use common::test_instr_exec_ok;

fn price_all_zero() -> Price {
    let acc_key = AccKey {
        val: [0; 32]
    };

    let ema = Ema {
        val: 0,
        numer: 0,
        denom: 0
    };

    let price_info = PriceInfo {
        conf: 0,
        corp_act: CorpAction::NoCorpAct,
        price: 0,
        pub_slot: 0,
        status: PriceStatus::Unknown
    };

    let price_comp = PriceComp {
        agg: price_info,
        latest: price_info,
        publisher: acc_key
    };

    Price {
        magic: MAGIC,
        ver: VERSION_2,
        atype: AccountType::Price as u32,
        size: 0,
        ptype: PriceType::Price,
        expo: 0,
        num: 0,
        num_qt: 0,
        last_slot: 0,
        valid_slot: 0,
        twap: ema,
        twac: ema,
        drv1: 0,
        drv2: 0,
        prod: acc_key,
        next: acc_key,
        prev_slot: 0,
        prev_price: 0,
        prev_conf: 0,
        drv3: 0,
        agg: price_info,
        comp: [price_comp; 32]
    }
}


#[tokio::test]
async fn test_price_not_stale() {
    let mut price = price_all_zero();
    price.agg.status = PriceStatus::Trading;
    test_instr_exec_ok(instruction::price_status_check(&price, PriceStatus::Trading)).await;
}


#[tokio::test]
async fn test_price_stale() {
    let mut price = price_all_zero();
    price.agg.status = PriceStatus::Trading;
    // Value 100 will cause an overflow because this is bigger than Solana slot in the test suite (its ~1-5).
    // As the check will be 5u - 100u ~= 1e18 > MAX_SLOT_DIFFERENCE. It can only break when Solana slot in the test suite becomes 
    // between 100 and 100+MAX_SLOT_DIFFERENCE.
    price.agg.pub_slot = 100;
    test_instr_exec_ok(instruction::price_status_check(&price, PriceStatus::Unknown)).await;
}
