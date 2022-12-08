use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{ invoke_signed },
    system_instruction,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
    system_program::ID as SYSTEM_PROGRAM_ID,
};
use std::convert::TryInto;
// use super::structs;
use crate::{schemas::states::pool::{
    Pool,
    POOL_PDA_LEN,
    POOL_SEED,
}, common::POOL_ACCOUNT_TYPE};
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
    let system_program_account = next_account_info(accounts_iter)?;
    if account.owner != &SYSTEM_PROGRAM_ID {
        return Err(ContractError::NotASystemAccount.into());
    }
    let inst_data = PoolInitializationIns::try_from_slice(&instruction_data)?;
    let lamports_required = Rent::get()?.minimum_balance(POOL_PDA_LEN);
    let pool_id = &inst_data.id;
    let account_seeds: &[&[u8]; 2] = &[
        pool_id,
        POOL_SEED,
    ];
    let (expected_pda, bump) = Pubkey::find_program_address(account_seeds, program_id);
    if expected_pda != *pda_account.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    let signers_seeds: &[&[u8]; 3] = &[
        pool_id,
        POOL_SEED,
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
    // let clock = Clock::get()?;
    let mut pool_account_data = Pool::try_from_slice(&pda_account.data.borrow())?;
    pool_account_data.id = inst_data.id;
    pool_account_data.name = inst_data.name;
    pool_account_data.total_deposited_power = 0;
    pool_account_data.reward_period = inst_data.reward_period;
    pool_account_data.account_type = POOL_ACCOUNT_TYPE;
    // pool_account_data.start_at = clock.unix_timestamp as u64;
    pool_account_data.start_at = inst_data.start_at;
    pool_account_data.creator = inst_data.creator;
    pool_account_data.collection = inst_data.collection;
    pool_account_data.pool_type = inst_data.pool_type;
    pool_account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    Ok(())
}
