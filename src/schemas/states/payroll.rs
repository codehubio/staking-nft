use borsh::{
  BorshSerialize,
  BorshDeserialize
};
use solana_program::{
  pubkey::Pubkey
};

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub struct Payroll {
    pub account_type: u8,
    pub total_deposited_power: u64,
    pub index: u64,
    pub number_of_reward_tokens: u64,
    pub claimable_after: u64,
    pub start_at: u64,
    pub pool_pda_account: Pubkey,
    pub creator: Pubkey,
}
pub const PAYROLL_PDA_LEN: usize = 1 + 8 + 8 + 8 + 8 + 8 + 32 + 32;
pub const PAYROLL_SEED: &[u8] = b"payroll";