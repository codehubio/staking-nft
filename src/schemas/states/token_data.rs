
use borsh::{
  BorshSerialize,
  BorshDeserialize
};

use solana_program::{
  pubkey::Pubkey
};


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TokenData {
  pub power: u64,
  pub token_mint_address: Pubkey,
}
// 73
pub const TOKEN_DATA_PDA_LEN: usize = 8 + 32;
pub const TOKEN_DATA_SEED: &[u8] = b"tokendata";