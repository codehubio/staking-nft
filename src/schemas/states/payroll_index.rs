use borsh::{
  BorshSerialize,
  BorshDeserialize
};
use solana_program::{
  pubkey::Pubkey
};

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub struct PayrollIndex {
    pub account_type: u8,
    pub reward_token_mint_account: Pubkey,
    pub index: u64,
    pub payroll_pda: Pubkey,
    pub creator: Pubkey,
}
pub const PAYROLL_INDEX_PDA_LEN: usize = 1 + 32 + 8 + 32 + 32;
pub const PAYROLL_INDEX_SEED: &[u8] = b"payrollindex";