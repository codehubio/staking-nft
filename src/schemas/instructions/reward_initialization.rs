use borsh::{
  BorshSerialize,
  BorshDeserialize,
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RewardInitialization {
  // name, 16 char
  pub index: u8,

}