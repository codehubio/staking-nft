use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{ invoke_signed, invoke },
    system_instruction,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
    system_program::ID as SYSTEM_PROGRAM_ID,
};
use spl_associated_token_account::{
    instruction as spl_instruction,
};
use std::convert::TryInto;
// use super::structs;
use crate::schemas::states::pool::{
    Pool,
    POOL_PDA_LEN,
    POOL_SEED,
};
use crate::schemas::instructions::pool_initialization::PoolInitializationIns;
use crate::error::ContractError;
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let reward_token_mint_account= next_account_info(accounts_iter)?;
    let reward_token_associated_account= next_account_info(accounts_iter)?;
    let token_program_account = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;
    if account.owner != &SYSTEM_PROGRAM_ID {
        return Err(ContractError::NotASystemAccount.into());
    }
    let inst_data = PoolInitializationIns::try_from_slice(&instruction_data)?;
    let lamports_required = Rent::get()?.minimum_balance(POOL_PDA_LEN);
    let pool_name = &inst_data.name;
    let account_seeds: &[&[u8]; 3] = &[
        pool_name,
        POOL_SEED,
        &account.key.to_bytes(),
    ];
    let (expected_pda, bump) = Pubkey::find_program_address(account_seeds, program_id);
    if expected_pda != *pda_account.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    
    let signers_seeds: &[&[u8]; 4] = &[
        pool_name,
        POOL_SEED,
        &account.key.to_bytes(),
        &[bump],
    ];
    let create_pda_account_ix = system_instruction::create_account(
        &account.key,
        &pda_account.key,
        lamports_required,
        POOL_PDA_LEN.try_into().unwrap(),
        &program_id,
    );
    
    invoke_signed(
        &create_pda_account_ix,
        &[
            account.clone(),
            pda_account.clone(),
            system_program_account.clone(),
        ],
        &[signers_seeds],
    )?;
    // create ata for reward
    let create_token_account_ix = spl_instruction::create_associated_token_account(
        &account.key,
        &pda_account.key,
        &reward_token_mint_account.key,
        &token_program_account.key
    );
    invoke(
        &create_token_account_ix,
        &[
            account.clone(),
            reward_token_associated_account.clone(),
            pda_account.clone(),
            reward_token_mint_account.clone(),
            system_program_account.clone(),
            token_program_account.clone(),
        ],
    )?;
    // let clock = Clock::get()?;
    let mut pool_account_data = Pool::try_from_slice(&pda_account.data.borrow())?;
    pool_account_data.name = inst_data.name;
    pool_account_data.total_deposited_power = 0;
    pool_account_data.reward_token_mint_address = *reward_token_mint_account.key;
    pool_account_data.reward_ata = *reward_token_associated_account.key;
    pool_account_data.reward_period = inst_data.reward_period;
    // pool_account_data.start_at = clock.unix_timestamp as u64;
    pool_account_data.start_at = inst_data.start_at;
    pool_account_data.creator = inst_data.creator;
    pool_account_data.collection = inst_data.collection;
    pool_account_data.pool_type = inst_data.pool_type;
    pool_account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    Ok(())
}
