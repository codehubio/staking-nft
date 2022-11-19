use borsh::{
  BorshSerialize,
  BorshDeserialize,
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TokenDataUpdate {
  
  pub token_power: u64,

}