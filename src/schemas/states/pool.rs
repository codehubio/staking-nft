use borsh::{
    BorshSerialize,
    BorshDeserialize
};

use solana_program::{
    pubkey::Pubkey
};


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Pool {
    // pool name 16 char
    pub name: [u8; 16],

    pub total_deposited_power: u64,
    // how frequently reward is calculated
    pub reward_period: u64,
    // start at
    pub start_at: u64,

    pub reward_token_mint_address: Pubkey,
    
    pub reward_ata: Pubkey,
    // poolType
    pub pool_type: u8,
    // creator
    pub creator: Pubkey,

    pub collection: Pubkey,

}
pub const POOL_PDA_LEN: usize = 16 + 8 + 8 + 8 + 32 + 32 + 1 + 32 + 32;
pub const POOL_SEED: &[u8] = b"pool";