use borsh::{
  BorshSerialize,
  BorshDeserialize
};

use solana_program::{
  pubkey::Pubkey
};

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub struct StakingAccount {
  pub account_type: u8,
  pub deposited_power: u64,
  pub deposited_at: u64,
  pub withdrawn_at: u64,
  pub first_payroll_index: u64,
  pub withdrawn_reward_amount: u64,
  pub pool_pda_account: Pubkey,
  pub withdrawn_address: Pubkey,
  pub staking_token_mint_address: Pubkey,
  pub depositor: Pubkey
}
pub const STAKING_PDA_LEN: usize = 1 + 8 + 8 + 8 + 8 + 8 + 32 + 32 + 32 + 32;
pub const STAKING_SEED: &[u8] = b"staking";