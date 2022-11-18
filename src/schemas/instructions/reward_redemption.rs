use borsh::{
  BorshSerialize,
  BorshDeserialize,
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RewardRedemption {
  // name, 16 char
  pub index: u64,

}