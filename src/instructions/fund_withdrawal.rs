use crate::common::{
    get_current_payroll_index, get_or_create_current_payroll,
    recalculate_reward_rate, verify_ata_account,
    verify_program_account, verify_system_account,
};
use crate::error::ContractError;
use crate::schemas::states::payroll::Payroll;
/// Define the type of state stored in accounts
use crate::schemas::states::pool::Pool;
use crate::schemas::states::staking_account::{StakingAccount, STAKING_SEED};
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
    let main_account = next_account_info(accounts_iter)?;
    // check for account
    // let pool_pda_account_data = pool_pda_account.data.borrow();
    verify_system_account(&account)?;
    verify_program_account(pool_pda_account, program_id)?;
    verify_program_account(pda_account, program_id)?;
    verify_program_account(staking_token_data_pda, program_id)?;
    let token_data_seeeds = &[
        TOKEN_DATA_SEED,
        &staking_token_mint_account.key.to_bytes(),
    ];
    let (expected_token_data_pda, _bump) = Pubkey::find_program_address(token_data_seeeds, program_id);
    if expected_token_data_pda != *staking_token_data_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    let token_data = TokenData::try_from_slice(&staking_token_data_pda.data.borrow())?;
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
    let mut withdrawn_amount = 1;
    // early withdrawl results in penalty
    msg!("Checking pool pda");
    if pda_account_data.pool_pda_account != *pool_pda_account.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }

    pda_account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    updated_pool_data.total_deposited_power -= token_data.power;
    updated_pool_data.serialize(&mut &mut pool_pda_account.data.borrow_mut()[..])?;
    let current_payroll_index = get_current_payroll_index(
        clock.unix_timestamp as u64,
        updated_pool_data.reward_period,
        updated_pool_data.start_at,
    );
    // now transfer
    let ata_dest_account_data_len = staking_token_dest_associated_account.data_len();
    // let pool_seeds: &[&[u8]; 3] = &[
    //   &updated_pool_data.name[..],
    //   POOL_SEED,
    //   &pool_creator_account.key.to_bytes(),
    // ];
    let pda_account_seeds: &[&[u8]; 4] = &[
        STAKING_SEED,
        &staking_token_mint_account.key.to_bytes(),
        &account.key.to_bytes(),
        &pool_pda_account.key.to_bytes()
    ];
    let (_, bump) = Pubkey::find_program_address(pda_account_seeds, program_id);
    // let pool_signers_seeds: &[&[u8]; 4] = &[
    //   &updated_pool_data.name[..],
    //     POOL_SEED,
    //     &pool_creator_account.key.to_bytes(),
    //     &[bump],
    // ];
    let pda_signers_seeds: &[&[u8]; 5] = &[
        STAKING_SEED,
        &staking_token_mint_account.key.to_bytes(),
        &account.key.to_bytes(),
        &pool_pda_account.key.to_bytes(),
        &[bump],
    ];
    // msg!("ata dst address: {:?}, {:?}" ,staking_account.withdrawn_address, dst_account.key);
    if ata_dest_account_data_len <= 0 {
        let create_token_account_ix = spl_instruction::create_associated_token_account(
            &main_account.key,
            &withdrawn_address,
            &staking_token_mint_account.key,
            &token_program_account.key,
        );
        invoke(
            &create_token_account_ix,
            &[
                main_account.clone(),
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
        &pda_account.key,
        &[],
        withdrawn_amount,
    )?;
    invoke_signed(
        &ix,
        &[
            staking_token_source_associated_account.clone(),
            staking_token_dest_associated_account.clone(),
            pda_account.clone(),
            token_program_account.clone(),
        ],
        &[pda_signers_seeds],
    )?;
    
    match get_or_create_current_payroll(
        program_id,
        main_account,
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
        let rate_reward = recalculate_reward_rate(
            current_payroll_data.total_deposited_power,
            current_payroll_data.total_reward_amount,
        );
        current_payroll_data.rate_reward = rate_reward;
        current_payroll_data.serialize(&mut &mut payroll_pda.data.borrow_mut()[..])?;

    }
    Ok(())
}
