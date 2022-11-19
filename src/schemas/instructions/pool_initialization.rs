use borsh::{
  BorshSerialize,
  BorshDeserialize,
};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PoolInitializationIns {
  
  pub name: [u8; 16],
  
  pub penalty_rate: u64,
   
  pub staking_duration: u64,
  
  pub reward_period: u64,

  pub start_at: u64,

  pub creator: Pubkey,

  pub collection: Pubkey,

  pub pool_type: u8,

}