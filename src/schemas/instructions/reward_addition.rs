use borsh::{
  BorshSerialize,
  BorshDeserialize,
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RewardAddition {
  
  pub amount: u64,

  pub payroll_index: u64,

}