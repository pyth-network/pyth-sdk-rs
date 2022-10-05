//! Program states
//! A data account would store an AdminConfig structure for instructions.
//! This file contains the serialization / deserialization of AdminConfig.

use solana_program::program_error::ProgramError;
use solana_program::program_pack::{
    IsInitialized,
    Pack,
    Sealed,
};
use solana_program::pubkey::Pubkey;

use arrayref::{
    array_mut_ref,
    array_ref,
    array_refs,
    mut_array_refs,
};

// loan_price_feed_id and collateral_price_feed_id are the
// Pyth price accounts for the loan and collateral tokens
pub struct AdminConfig {
    pub is_initialized:           bool,
    pub loan_price_feed_id:       Pubkey,
    pub collateral_price_feed_id: Pubkey,
}

impl Sealed for AdminConfig {
}

impl IsInitialized for AdminConfig {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for AdminConfig {
    const LEN: usize = 1 + 32 + 32;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, AdminConfig::LEN];
        let (src_is_initialized, src_loan_price_feed_id, src_collateral_price_feed_id) =
            array_refs![src, 1, 32, 32];

        let is_initialized = match src_is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(AdminConfig {
            is_initialized,
            loan_price_feed_id: Pubkey::new_from_array(*src_loan_price_feed_id),
            collateral_price_feed_id: Pubkey::new_from_array(*src_collateral_price_feed_id),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, AdminConfig::LEN];
        let (dst_is_initialized, dst_loan_price_feed_id, dst_collateral_price_feed_id) =
            mut_array_refs![dst, 1, 32, 32];

        let AdminConfig {
            is_initialized,
            loan_price_feed_id,
            collateral_price_feed_id,
        } = self;

        dst_is_initialized[0] = *is_initialized as u8;
        dst_loan_price_feed_id.copy_from_slice(loan_price_feed_id.as_ref());
        dst_collateral_price_feed_id.copy_from_slice(collateral_price_feed_id.as_ref());
    }
}
