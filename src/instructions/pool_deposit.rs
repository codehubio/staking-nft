use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{ invoke_signed, invoke },
    system_instruction,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
    clock::Clock,
    msg,
};
use std::{
    convert::TryInto
};
use crate::{common::{   
    get_or_create_next_payroll_by_time,
    verify_system_account,
    verify_program_account,
    get_staking_pda, STAKING_ACCOUNT_TYPE, POOL_PAYROLL_ACCOUNT_TYPE, TOKEN_DATA_ACCOUNT_TYPE,
}, schemas::states::token_data::TOKEN_DATA_PDA_LEN};

/// Define the type of state stored in accounts
use crate::schemas::states::pool::{
    Pool,
};
use crate::schemas::states::payroll::{
    Payroll,
};

use mpl_token_metadata::{ID as MPL_PROGRAM_ID, state::TokenMetadataAccount};
use mpl_token_metadata::state::Metadata;

use crate::schemas::states::staking_account::{
    StakingAccount,
    STAKING_PDA_LEN,
    STAKING_SEED
};

use crate::schemas::states::token_data::{
    TokenData,
    TOKEN_DATA_SEED
};

use spl_associated_token_account::{
    instruction as spl_instruction,
};
use crate::error::ContractError;
pub fn process_instruction <'a>(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &'a [AccountInfo<'a>], // The account to say hello to
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let pool_pda_account = next_account_info(accounts_iter)?;
    let staking_token_mint_account= next_account_info(accounts_iter)?;
    let staking_token_source_associated_account= next_account_info(accounts_iter)?;
    let staking_token_dest_associated_account = next_account_info(accounts_iter)?;
    let staking_token_data_pda = next_account_info(accounts_iter)?;
    let payroll_pda = next_account_info(accounts_iter)?;
    let meta_pda = next_account_info(accounts_iter)?;
    let token_program_account = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;
    // check for account
    // let pool_pda_account_data = pool_pda_account.data.borrow();
    verify_system_account(account)?;
    verify_program_account(pool_pda_account, program_id)?;
    // verify_program_account(staking_token_data_pda, program_id)?;
    let token_data_seeeds = &[
        TOKEN_DATA_SEED,
        &staking_token_mint_account.key.to_bytes(),
    ];
    if *meta_pda.owner != MPL_PROGRAM_ID {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    let metadata = Metadata::from_account_info(meta_pda)?;
    let (expected_token_data_pda, _bump) = Pubkey::find_program_address(token_data_seeeds, program_id);
    if expected_token_data_pda != *staking_token_data_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    // let inst_data = PoolDepositIns::try_from_slice(&instruction_data)?;
    let mut pool_data = Pool::try_from_slice(&pool_pda_account.data.borrow())?;
    let collection = metadata.collection.unwrap();
    if collection.key != pool_data.collection || collection.verified != true {
        return Err(ContractError::InvalidCollection.into());
    }
    // accept +- 10 seconds differences
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    let (expected_pda_account, bump) = get_staking_pda(
        &pool_pda_account.key,
        &account.key,
        &staking_token_mint_account.key,
        program_id
    ).ok().unwrap();
    let token_data: TokenData = match staking_token_data_pda.data_len() != TOKEN_DATA_PDA_LEN {
        true => TokenData {
            account_type: TOKEN_DATA_ACCOUNT_TYPE,
            power: 1,
            token_mint_address: *staking_token_mint_account.key,
        },
        false => TokenData::try_from_slice(&staking_token_data_pda.data.borrow())?
    };
    // let token_data = TokenData::try_from_slice(&staking_token_data_pda.data.borrow())?;
    let (next_payroll, next_payroll_index) = match get_or_create_next_payroll_by_time(
        now as u64,
        program_id,
        account,
        pool_pda_account,
        payroll_pda,
        system_program_account,
        pool_data.clone(),
    ) {
        Ok(p) => p,
        Err(err) => return Err(err),
    };
    if next_payroll != *payroll_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    if *pda_account.key != expected_pda_account {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    let first_payroll_index = next_payroll_index;
    msg!("Checking for previous deposit");
    let lamports_required = Rent::get()?.minimum_balance(STAKING_PDA_LEN);
    let withdrawn_address = *account.key;
    let deposited_at = clock.unix_timestamp as u64;
    let signers_seeds: &[&[u8]; 5] = &[
        STAKING_SEED,
        &staking_token_mint_account.key.to_bytes(),
        &account.key.to_bytes(),
        &pool_pda_account.key.to_bytes(),
        &[bump],
    ];
    let pda_account_data_len = pda_account.data_len();
    if pda_account_data_len <= 0 {
        msg!("Creating or updating pda");
        let create_pda_account_ix = system_instruction::create_account(
            &account.key,
            &pda_account.key,
            lamports_required,
            STAKING_PDA_LEN.try_into().unwrap(),
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
        let create_token_account_ix = spl_instruction::create_associated_token_account(
            &account.key,
            &pool_pda_account.key,
            &staking_token_mint_account.key,
            // &token_program_account.key
        );
        invoke(
            &create_token_account_ix,
            &[
              account.clone(),
              staking_token_dest_associated_account.clone(),
              pool_pda_account.clone(),
              staking_token_mint_account.clone(),
              system_program_account.clone(),
              token_program_account.clone(),
            ],
        )?;

    }
    // now transfer
    let ix = spl_token::instruction::transfer(
        &token_program_account.key,
        &staking_token_source_associated_account.key,
        &staking_token_dest_associated_account.key,
        &account.key,
        &[],
        1,
    )?;
    // let signers_seeds: &[&[u8]; 1] = &[
    //     &pda_account.key.to_bytes(),
    // ];
    match invoke(
        &ix,
        &[
            staking_token_source_associated_account.clone(),
            staking_token_dest_associated_account.clone(),
            account.clone(),
            token_program_account.clone(),
        ],
    ) {
        Ok(p) => p,
        Err(_err) => return Err(ContractError::TransferError.into()),
    };
    let staking_account = StakingAccount {
        account_type: STAKING_ACCOUNT_TYPE,
        deposited_power: token_data.power,
        deposited_at,
        withdrawn_at: 0,
        withdrawn_reward_amount: 0,
        first_payroll_index,
        depositor: account.key.clone(),
        pool_pda_account: pool_pda_account.key.clone(),
        staking_token_mint_address: *staking_token_mint_account.key,
        withdrawn_address,
    };
    staking_account.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    pool_data.total_deposited_power += token_data.power;
    let reward_period = pool_data.reward_period;
    let start_at = pool_data.start_at;
    let total_deposited_power = pool_data.total_deposited_power;
    let mut number_of_reward_tokens = 0;
    pool_data.serialize(&mut &mut pool_pda_account.data.borrow_mut()[..])?;
    if payroll_pda.data_len() > 0 {
        let current_payroll_data = Payroll::try_from_slice(&payroll_pda.data.borrow())?;
        number_of_reward_tokens = current_payroll_data.number_of_reward_tokens;
    }
    let payroll_account_data = Payroll {
        account_type: POOL_PAYROLL_ACCOUNT_TYPE,
        number_of_reward_tokens,
        total_deposited_power,
        index: next_payroll_index,
        start_at,
        claimable_after: start_at + next_payroll_index * reward_period,
        pool_pda_account: *pool_pda_account.key,
        creator: *account.key
    };
    payroll_account_data.serialize(&mut &mut payroll_pda.data.borrow_mut()[..])?;

    Ok(())
}
