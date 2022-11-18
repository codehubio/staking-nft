use borsh::{
  BorshSerialize,
  BorshDeserialize,
};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PoolDepositIns {
  // url 
  pub withdrawn_address: Pubkey,

}