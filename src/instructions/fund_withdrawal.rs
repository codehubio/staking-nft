use crate::common::{
    get_or_create_current_payroll,
    verify_program_account, verify_system_account, TOKEN_DATA_ACCOUNT_TYPE,
};
use crate::error::ContractError;
use crate::schemas::states::payroll::Payroll;
/// Define the type of state stored in accounts
use crate::schemas::states::pool::{Pool, POOL_SEED};
use crate::schemas::states::staking_account::{StakingAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use crate::schemas::states::token_data::{
    TokenData,
    TOKEN_DATA_SEED
};


use spl_associated_token_account::instruction as spl_instruction;
// Program entrypoint's implementation
pub fn process_instruction<'a>(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &'a [AccountInfo<'a>], // The account to say hello to
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let pool_pda_account = next_account_info(accounts_iter)?;
    let withdraw_account = next_account_info(accounts_iter)?;
    let staking_token_mint_account = next_account_info(accounts_iter)?;
    let staking_token_source_associated_account = next_account_info(accounts_iter)?;
    let staking_token_dest_associated_account = next_account_info(accounts_iter)?;
    let staking_token_data_pda = next_account_info(accounts_iter)?;
    let payroll_pda = next_account_info(accounts_iter)?;
    let token_program_account = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;
    // check for account
    // let pool_pda_account_data = pool_pda_account.data.borrow();
    verify_system_account(&account)?;
    verify_program_account(pool_pda_account, program_id)?;
    verify_program_account(pda_account, program_id)?;
    // verify_program_account(staking_token_data_pda, program_id)?;
    let token_data_seeeds = &[
        TOKEN_DATA_SEED,
        &staking_token_mint_account.key.to_bytes(),
    ];
    let (expected_token_data_pda, _bump) = Pubkey::find_program_address(token_data_seeeds, program_id);
    if expected_token_data_pda != *staking_token_data_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    let token_data = match staking_token_data_pda.owner != program_id {
        true => TokenData {
            account_type: TOKEN_DATA_ACCOUNT_TYPE,
            power: 1,
            token_mint_address: *staking_token_mint_account.key,
        },
        false => TokenData::try_from_slice(&staking_token_data_pda.data.borrow())?
    };
  
    // let token_data = TokenData::try_from_slice(&staking_token_data_pda.data.borrow())?;
    let mut updated_pool_data = Pool::try_from_slice(&pool_pda_account.data.borrow())?;
    let mut pda_account_data = StakingAccount::try_from_slice(&pda_account.data.borrow())?;
    let withdrawn_address = pda_account_data.withdrawn_address;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp as u64;
    msg!("Checking if fund withdrawn");
    if pda_account_data.withdrawn_at > 0 {
        return Err(ContractError::FundAlreadyWithdrawn.into());
    }
    let depositor = pda_account_data.depositor;
    // not valid withdrawal
    msg!("Checking owner");
    if !account.is_signer || *account.key != depositor {
        return Err(ContractError::InvalidDepositor.into());
    }
    msg!("Checking withdrawn address");
    if pda_account_data.withdrawn_address != *withdraw_account.key {
        return Err(ContractError::InvalidWithdrawnAddress.into());
    }
    pda_account_data.withdrawn_at = now;
    // early withdrawl results in penalty
    msg!("Checking pool pda");
    if pda_account_data.pool_pda_account != *pool_pda_account.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }

    pda_account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    updated_pool_data.total_deposited_power -= token_data.power;
    updated_pool_data.serialize(&mut &mut pool_pda_account.data.borrow_mut()[..])?;
    // now transfer
    let ata_dest_account_data_len = staking_token_dest_associated_account.data_len();
    // let pool_seeds: &[&[u8]; 3] = &[
    //   &updated_pool_data.name[..],
    //   POOL_SEED,
    //   &pool_creator_account.key.to_bytes(),
    // ];
    let pool_pda_account_seeds: &[&[u8]; 2] = &[
        &updated_pool_data.id[..],
        POOL_SEED,
    ];
    let (_, bump) = Pubkey::find_program_address(pool_pda_account_seeds, program_id);
    // let pool_signers_seeds: &[&[u8]; 4] = &[
    //   &updated_pool_data.name[..],
    //     POOL_SEED,
    //     &pool_creator_account.key.to_bytes(),
    //     &[bump],
    // ];
    let pool_pda_signers_seeds: &[&[u8]; 3] = &[
        &updated_pool_data.id[..],
        POOL_SEED,
        &[bump],
    ];
    // msg!("ata dst address: {:?}, {:?}" ,staking_account.withdrawn_address, dst_account.key);
    if ata_dest_account_data_len <= 0 {
        let create_token_account_ix = spl_instruction::create_associated_token_account(
            &account.key,
            &withdrawn_address,
            &staking_token_mint_account.key,
            // &token_program_account.key,
        );
        invoke(
            &create_token_account_ix,
            &[
                account.clone(),
                staking_token_dest_associated_account.clone(),
                withdraw_account.clone(),
                staking_token_mint_account.clone(),
                system_program_account.clone(),
                token_program_account.clone(),
            ],
        )?;
    }
    let ix = spl_token::instruction::transfer(
        &token_program_account.key,
        &staking_token_source_associated_account.key,
        &staking_token_dest_associated_account.key,
        &pool_pda_account.key,
        &[],
        1,
    )?;
    invoke_signed(
        &ix,
        &[
            staking_token_source_associated_account.clone(),
            staking_token_dest_associated_account.clone(),
            pool_pda_account.clone(),
            token_program_account.clone(),
        ],
        &[pool_pda_signers_seeds],
    )?;
    
    match get_or_create_current_payroll(
        program_id,
        account,
        pool_pda_account,
        payroll_pda,
        system_program_account,
        updated_pool_data.clone(),
    ) {
        Ok(p) => p,
        Err(err) => return Err(err),
    };
    if payroll_pda.data_len() > 0 {
        let mut current_payroll_data = Payroll::try_from_slice(&payroll_pda.data.borrow())?;
        current_payroll_data.total_deposited_power = updated_pool_data.total_deposited_power;
        current_payroll_data.serialize(&mut &mut payroll_pda.data.borrow_mut()[..])?;

    }
    Ok(())
}
