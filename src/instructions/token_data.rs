use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    msg,
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{ invoke_signed },
    system_instruction,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use std::{
    convert::TryInto
};
use crate::common::{
    verify_system_account, TOKEN_DATA_ACCOUNT_TYPE,
};

/// Define the type of state stored in accounts
use crate::schemas::instructions::{
    token_data,
};


use crate::schemas::states::token_data::{
    TokenData,
    TOKEN_DATA_SEED,
    TOKEN_DATA_PDA_LEN
};

use crate::error::ContractError;
pub fn process_instruction <'a>(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &'a [AccountInfo<'a>], // The account to say hello to
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    let staking_token_mint_account= next_account_info(accounts_iter)?;
    let staking_token_data_pda = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;
    // check for account
    // let pool_pda_account_data = pool_pda_account.data.borrow();
    msg!("Verifying accounts");
    let inst = token_data::TokenDataUpdate::try_from_slice(instruction_data)?;
    verify_system_account(account)?;
    let token_data_seeeds = &[
        TOKEN_DATA_SEED,
        &staking_token_mint_account.key.to_bytes(),
    ];
    let (expected_token_data_pda, bump) = Pubkey::find_program_address(token_data_seeeds, program_id);
    if expected_token_data_pda != *staking_token_data_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    let token_data_signer_seeeds = &[
        TOKEN_DATA_SEED,
        &staking_token_mint_account.key.to_bytes(),
        &[bump]
    ];
    let lamports_required = Rent::get()?.minimum_balance(TOKEN_DATA_PDA_LEN);
    let create_pda_account_ix = system_instruction::create_account(
        &account.key,
        &staking_token_data_pda.key,
        lamports_required,
        TOKEN_DATA_PDA_LEN.try_into().unwrap(),
        &program_id,
    );
    
    invoke_signed(
        &create_pda_account_ix,
        &[
            account.clone(),
            staking_token_data_pda.clone(),
            system_program_account.clone(),
        ],
        &[token_data_signer_seeeds],
    )?;
    let token_data = TokenData {
        account_type: TOKEN_DATA_ACCOUNT_TYPE,
        power: inst.token_power,
        token_mint_address: *staking_token_mint_account.key,
    };
    token_data.serialize(&mut &mut staking_token_data_pda.data.borrow_mut()[..])?;
    Ok(())
}
