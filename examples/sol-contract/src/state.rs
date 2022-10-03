use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub struct LoanInfo {
    pub is_initialized: bool,
    pub loan_key: Pubkey,
    pub loan_qty: i64,
    pub collateral_key: Pubkey,
    pub collateral_qty: i64
}

impl Sealed for LoanInfo {}

impl IsInitialized for LoanInfo {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for LoanInfo {
    const LEN: usize = 1 + 32 + 8 + 32 + 8;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, LoanInfo::LEN];
        let (
            src_is_initialized,
            src_loan_key, src_loan_qty,
            src_collateral_key, src_collateral_qty,
        ) = array_refs![src, 1, 32, 8, 32, 8];
        let is_initialized = match src_is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(LoanInfo {
            is_initialized,
            loan_key: Pubkey::new_from_array(*src_loan_key),
            loan_qty: i64::from_le_bytes(*src_loan_qty),
            collateral_key: Pubkey::new_from_array(*src_collateral_key),
            collateral_qty: i64::from_le_bytes(*src_collateral_qty),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, LoanInfo::LEN];
        let (
            dst_is_initialized,
            dst_loan_key, dst_loan_qty,
            dst_collateral_key, dst_collateral_qty,
        ) = mut_array_refs![dst, 1, 32, 8, 32, 8];

        let LoanInfo {
            is_initialized,
            loan_key, loan_qty,
            collateral_key, collateral_qty,
        } = self;

        dst_is_initialized[0] = *is_initialized as u8;
        dst_loan_key.copy_from_slice(loan_key.as_ref());
        *dst_loan_qty = loan_qty.to_le_bytes();
        dst_collateral_key.copy_from_slice(collateral_key.as_ref());
        *dst_collateral_qty = collateral_qty.to_le_bytes();
    }
}
