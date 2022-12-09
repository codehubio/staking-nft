
use borsh::{
  BorshSerialize,
  BorshDeserialize
};

use solana_program::{
  pubkey::Pubkey
};


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CollectionData {
  pub account_type: u8,
  pub collection_mint_address: Pubkey,
}
// 73
pub const COLLECTION_DATA_PDA_LEN: usize = 1 + 32;
pub const COLLECTION_DATA_SEED: &[u8] = b"collectiondata";