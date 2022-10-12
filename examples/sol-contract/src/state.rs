//! Program states
//! A data account would store an AdminConfig structure for instructions.
//! This file contains the serialization / deserialization of AdminConfig.

use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use solana_program::pubkey::Pubkey;

// loan_price_feed_id and collateral_price_feed_id are the
// Pyth price accounts for the loan and collateral tokens
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct AdminConfig {
    pub is_initialized:           bool,
    pub loan_price_feed_id:       Pubkey,
    pub collateral_price_feed_id: Pubkey,
}
