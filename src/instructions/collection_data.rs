use borsh::{BorshSerialize};
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
use crate::{common::{
    verify_system_account, COLLECTION_DATA_ACCOUNT_TYPE,
}};

/// Define the type of state stored in accounts


use crate::schemas::states::collection_data::{
    CollectionData,
    COLLECTION_DATA_SEED,
    COLLECTION_DATA_PDA_LEN
};

use crate::error::ContractError;
pub fn process_instruction <'a>(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &'a [AccountInfo<'a>], // The account to say hello to
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    let collection_mint_account= next_account_info(accounts_iter)?;
    let collection_data_pda = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;
    // check for account
    // let pool_pda_account_data = pool_pda_account.data.borrow();
    msg!("Verifying accounts");
    verify_system_account(account)?;
    let collection_data_seeeds = &[
        COLLECTION_DATA_SEED,
        &collection_mint_account.key.to_bytes(),
    ];
    let (expected_collection_data_pda, bump) = Pubkey::find_program_address(collection_data_seeeds, program_id);
    if expected_collection_data_pda != *collection_data_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    let collection_data_signer_seeeds = &[
        COLLECTION_DATA_SEED,
        &collection_mint_account.key.to_bytes(),
        &[bump]
    ];
    let lamports_required = Rent::get()?.minimum_balance(COLLECTION_DATA_PDA_LEN);
    let create_pda_account_ix = system_instruction::create_account(
        &account.key,
        &collection_data_pda.key,
        lamports_required,
        COLLECTION_DATA_PDA_LEN.try_into().unwrap(),
        &program_id,
    );
    
    invoke_signed(
        &create_pda_account_ix,
        &[
            account.clone(),
            collection_data_pda.clone(),
            system_program_account.clone(),
        ],
        &[collection_data_signer_seeeds],
    )?;
    let collection_data = CollectionData {
        account_type: COLLECTION_DATA_ACCOUNT_TYPE,
        collection_mint_address: *collection_mint_account.key,
    };
    collection_data.serialize(&mut &mut collection_data_pda.data.borrow_mut()[..])?;
    Ok(())
}
