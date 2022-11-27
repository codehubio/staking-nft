
use borsh::{
  BorshSerialize,
  BorshDeserialize
};

use solana_program::{
  pubkey::Pubkey
};


#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub struct StakingPayroll {
    pub account_type: u8,
    pub staking_pda_account: Pubkey,
    pub deposited_power: u64,
    pub total_pool_deposited_power: u64,
    pub total_reward_amount: u64,
    pub reward_withdrawn_amount: u64,
    pub index: u64,
    pub withdrawn_at: u64,
}
pub const STAKING_PAYROLL_SEED: &[u8] = b"stakingpayroll";
pub const STAKING_PAYROLL_PDA_LEN: usize = 1 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8;