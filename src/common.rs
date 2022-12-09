use crate::schemas::states::pool::{
    Pool,
};
use crate::schemas::states::payroll::{
    PAYROLL_SEED,
    PAYROLL_PDA_LEN
};

use crate::schemas::states::staking_account::{
    STAKING_SEED,
};
use solana_program::{
    clock::Clock,
    sysvar::Sysvar,
    program_error::ProgramError,
    program::{ invoke_signed },
    rent::Rent,
    pubkey::Pubkey, account_info::AccountInfo,
    system_instruction,
    system_program::ID as SYSTEM_PROGRAM_ID,
    msg,
};
use spl_associated_token_account::{
    get_associated_token_address
};
use std::{
    convert::TryInto
};
use crate::error::ContractError::{
    InvalidProgramAccount,
    InvalidAtaAccount,
};

pub const DECIMAL_REWARD: u32 = 6;
pub const POOL_ACCOUNT_TYPE: u8 = 100;
pub const STAKING_ACCOUNT_TYPE: u8 = 101;
pub const POOL_PAYROLL_ACCOUNT_TYPE: u8 = 102;
pub const POOL_PAYROLL_TOKEN_ACCOUNT_TYPE: u8 = 103;
pub const POOL_PAYROLL_INDEX_ACCOUNT_TYPE: u8 = 104;
pub const STAKING_PAYROLL_ACCOUNT_TYPE: u8 = 105;
pub const TOKEN_DATA_ACCOUNT_TYPE: u8 = 106;
pub const COLLECTION_DATA_ACCOUNT_TYPE: u8 = 107;

pub fn get_current_payroll_index(
    current_at: u64,
    reward_period: u64,
    start_at: u64
) -> u64 {
    ((current_at - start_at) / reward_period) + 1
}

pub fn get_or_create_payroll_by_index <'a>(
    payroll_index: u64,
    program_id: &Pubkey,
    main_account: &'a AccountInfo <'a>,
    pool_pda_account: &'a AccountInfo<'a>,
    payroll_pda: &'a AccountInfo<'a>,
    system_program_account: &'a AccountInfo <'a>,
) -> Result<(Pubkey, u64), ProgramError> {
    let parsed_index = payroll_index.to_string();
    let payroll_account_seeds: &[&[u8]; 3] = &[
        PAYROLL_SEED,
        parsed_index.as_bytes(),
        &pool_pda_account.key.to_bytes(),
    ];
    let (pda_payroll_key, pbump) = Pubkey::find_program_address(&payroll_account_seeds[..], program_id);
    // msg!("Incorrect payroll account, index: {:?}, expected {:?}, found {:?}", parsed_index, pda_payroll_key, *payroll_pda.key);
    if pda_payroll_key != *payroll_pda.key {
        return Err(ProgramError::InvalidAccountData);
    }
    let payroll_signer_seeds: &[&[u8]; 4] = &[
        PAYROLL_SEED,
        parsed_index.as_bytes(),
        &pool_pda_account.key.to_bytes(),
        &[pbump],
    ];
    let pda_payroll_account_data_len = payroll_pda.data_len();
    let payroll_lamports_required = Rent::get().ok().unwrap().minimum_balance(PAYROLL_PDA_LEN);
    if pda_payroll_account_data_len <= 0 {
        let create_pda_account_ix = system_instruction::create_account(
            &main_account.key,
            &payroll_pda.key,
            payroll_lamports_required,
            PAYROLL_PDA_LEN.try_into().unwrap(),
            &program_id,
        );
        
        invoke_signed(
            &create_pda_account_ix,
            &[
                main_account.clone(),
                payroll_pda.clone(),
                system_program_account.clone(),
            ],
            &[payroll_signer_seeds],
        )?
    }
    Ok((pda_payroll_key, payroll_index))
}
pub fn get_or_create_next_payroll_by_time <'a>(
    now: u64,
    program_id: &Pubkey,
    main_account: &'a AccountInfo <'a>,
    pool_pda_account: &'a AccountInfo<'a>,
    payroll_pda: &'a AccountInfo<'a>,
    system_program_account: &'a AccountInfo <'a>,
    pool_data: Pool
) -> Result<(Pubkey, u64), ProgramError> {
    let next_payroll_index = get_current_payroll_index(
        now,
        pool_data.reward_period,
        pool_data.start_at,
    ) + 1;
    get_or_create_payroll_by_index(
        next_payroll_index,
        program_id,
        main_account,
        pool_pda_account,
        payroll_pda,
        system_program_account,
    )
}
pub fn get_or_create_next_payroll <'a>(
    program_id: &Pubkey,
    main_account: &'a AccountInfo <'a>,
    pool_pda_account: &'a AccountInfo<'a>,
    payroll_pda: &'a AccountInfo<'a>,
    system_program_account: &'a AccountInfo <'a>,
    pool_data: Pool
) -> Result<(Pubkey, u64), ProgramError> {
    let clock = Clock::get().ok().unwrap();
    get_or_create_next_payroll_by_time(
        clock.unix_timestamp as u64,
        program_id,
        main_account,
        pool_pda_account,
        payroll_pda,
        system_program_account,
        pool_data
    )
}

pub fn get_or_create_current_payroll_by_time <'a>(
    now: u64,
    program_id: &Pubkey,
    main_account: &'a AccountInfo <'a>,
    pool_pda_account: &'a AccountInfo<'a>,
    payroll_pda: &'a AccountInfo<'a>,
    system_program_account: &'a AccountInfo <'a>,
    pool_data: Pool
) -> Result<(Pubkey, u64), ProgramError> {
    let next_payroll_index = get_current_payroll_index(
        now,
        pool_data.reward_period,
        pool_data.start_at,
    );
    get_or_create_payroll_by_index(
        next_payroll_index,
        program_id,
        main_account,
        pool_pda_account,
        payroll_pda,
        system_program_account,
    )
}

pub fn get_or_create_current_payroll <'a>(
    program_id: &Pubkey,
    main_account: &'a AccountInfo <'a>,
    pool_pda_account: &'a AccountInfo<'a>,
    payroll_pda: &'a AccountInfo<'a>,
    system_program_account: &'a AccountInfo <'a>,
    pool_data: Pool
) -> Result<(Pubkey, u64), ProgramError> {
    let clock = Clock::get().ok().unwrap();
    get_or_create_current_payroll_by_time(
        clock.unix_timestamp as u64,
        program_id,
        main_account,
        pool_pda_account,
        payroll_pda,
        system_program_account,
        pool_data
    )
}



pub fn recalculate_reward_rate(
    total_deposited_power: u64,
    total_reward_amount: u64,
) -> u64 {

    match total_deposited_power {
        0 => 0,
        _ => total_reward_amount * u64::pow(10, DECIMAL_REWARD)/ total_deposited_power,
    }
}

pub fn verify_program_account(account: &AccountInfo, program_id: &Pubkey) -> Result<(), ProgramError> {
    msg!("{:?}, {:?}", account.owner, *program_id);
    match *account.owner == *program_id {
        true => Ok(()),
        false => return Err(InvalidProgramAccount.into()),
    }
}
pub fn verify_system_account(account: &AccountInfo) -> Result<(), ProgramError> {
    verify_program_account(account, &SYSTEM_PROGRAM_ID)
}
pub fn verify_ata_account(
    address: &Pubkey,
    ata: &Pubkey,
    mint: &Pubkey,
) -> Result<(), ProgramError> {
    let token_ata = get_associated_token_address(
        &address,
        mint,
    );
    if token_ata != *ata {
        return Err(InvalidAtaAccount.into());
    }
    Ok(())
}
pub fn get_staking_pda(
    pool_pda: &Pubkey,
    address: &Pubkey,
    mint: &Pubkey,
    program_id: &Pubkey,
) -> Result<(Pubkey, u8), ProgramError> {
    let account_seeds: &[&[u8]; 4] = &[
        STAKING_SEED,
        &mint.to_bytes(),
        &address.to_bytes(),
        &pool_pda.to_bytes()
    ];
    let (expected_pda_account, bump) = Pubkey::find_program_address(
        &account_seeds[..],
        program_id
    );

    Ok((expected_pda_account, bump))
}
