
use borsh::{
  BorshSerialize,
  BorshDeserialize
};

use solana_program::{
  pubkey::Pubkey
};


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RewardToken {
  pub account_type: u8,
  pub reward_token_mint_address: Pubkey,
  pub reward_ata: Pubkey,
}
// 73
pub const REWARD_TOKEN_PDA_LEN: usize = 1 + 1 + 32 + 32;
pub const REWARD_TOKEN_SEED: &[u8] = b"rewardtoken";