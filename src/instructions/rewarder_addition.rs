use std::convert::TryInto;

use crate::common::{
    get_or_create_payroll_by_index,
    verify_ata_account, verify_system_account, POOL_PAYROLL_ACCOUNT_TYPE, POOL_PAYROLL_TOKEN_ACCOUNT_TYPE, POOL_PAYROLL_INDEX_ACCOUNT_TYPE,
};
use crate::schemas::states::payroll_index::{PAYROLL_INDEX_SEED, PAYROLL_INDEX_PDA_LEN, PayrollIndex};
use crate::schemas::states::payroll_token::{PAYROLL_TOKEN_SEED, PAYROLL_TOKEN_PDA_LEN, PayrollToken};
use crate::schemas::states::pool::{Pool};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program::invoke_signed;
use solana_program::rent::Rent;
use solana_program::system_instruction;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    pubkey::Pubkey,
};
use spl_associated_token_account::{
    instruction as spl_instruction,
};
use crate::schemas::states::payroll::{Payroll};

use crate::schemas::instructions::reward_addition::RewardAddition;

use crate::error::ContractError;
pub fn process_instruction<'a>(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &'a [AccountInfo<'a>], // The account to say hello to
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    let pool_pda_account = next_account_info(accounts_iter)?;
    let reward_token_mint_account = next_account_info(accounts_iter)?;
    let reward_token_source_associated_account = next_account_info(accounts_iter)?;
    let reward_token_dest_associated_account = next_account_info(accounts_iter)?;
    let payroll_token_pda = next_account_info(accounts_iter)?;
    let payroll_index_pda = next_account_info(accounts_iter)?;
    let payroll_pda = next_account_info(accounts_iter)?;
    let token_program_account = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    verify_system_account(account)?;
    verify_ata_account(
        &account.key,
        reward_token_source_associated_account.key,
        &reward_token_mint_account.key,
    )?;
    verify_ata_account(
        &payroll_pda.key,
        reward_token_dest_associated_account.key,
        &reward_token_mint_account.key,
    )?;
    let inst_data = RewardAddition::try_from_slice(instruction_data)?;
    let current_payroll_index = inst_data.payroll_index;
    let parsed_current_payroll_index = current_payroll_index.to_string();
    let payroll_token_pda_account_seeds :&[&[u8]; 4] = &[
        PAYROLL_TOKEN_SEED,
        parsed_current_payroll_index.as_bytes(),
        &reward_token_mint_account.key.to_bytes(),
        &payroll_pda.key.to_bytes(),
    ];
    let (expected_payroll_token_pda, token_bump) = Pubkey::find_program_address(payroll_token_pda_account_seeds, program_id);
    if expected_payroll_token_pda != *payroll_token_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    let amount = inst_data.amount;
    let mut total_reward_tokens = 0;
    let mut number_of_reward_tokens: u64 = 1;
    let updated_pool_data = Pool::try_from_slice(&pool_pda_account.data.borrow())?;
    let reward_period = updated_pool_data.reward_period;
    let start_at = updated_pool_data.start_at;
    if payroll_pda.data_len() <= 0 {
        let (_current_payroll_pda, _currrent_payroll_index) = match get_or_create_payroll_by_index(
            current_payroll_index,
            program_id,
            account,
            pool_pda_account,
            payroll_pda,
            system_program_account,
        ) {
            Ok(p) => p,
            Err(err) => return Err(err),
        };
        let payroll_data = Payroll {
            account_type: POOL_PAYROLL_ACCOUNT_TYPE,
            total_deposited_power: updated_pool_data.total_deposited_power,
            index: current_payroll_index,
            number_of_reward_tokens,
            start_at,
            claimable_after: start_at + current_payroll_index * reward_period,
            pool_pda_account: *pool_pda_account.key,
            creator: *account.key,
        };
        payroll_data.serialize(&mut &mut payroll_pda.data.borrow_mut()[..])?;
        if reward_token_mint_account.key != system_program_account.key {
            let create_token_account_ix = spl_instruction::create_associated_token_account(
                &account.key,
                &payroll_pda.key,
                &reward_token_mint_account.key,
                // &token_program_account.key
            );
            invoke(
                &create_token_account_ix,
                &[
                  account.clone(),
                  reward_token_dest_associated_account.clone(),
                  payroll_pda.clone(),
                  reward_token_mint_account.clone(),
                  system_program_account.clone(),
                  token_program_account.clone(),
                ],
            )?; 
        }
    } else {
        let mut payroll_data = Payroll::try_from_slice(&payroll_pda.data.borrow())?;
        number_of_reward_tokens = payroll_data.number_of_reward_tokens + 1;
        payroll_data.number_of_reward_tokens += 1;
        payroll_data.serialize(&mut &mut payroll_token_pda.data.borrow_mut()[..])?;
    }
    let parsed_number_of_reward_tokens = number_of_reward_tokens.to_string();
    let payroll_index_pda_account_seeds :&[&[u8]; 3] = &[
        PAYROLL_INDEX_SEED,
        parsed_number_of_reward_tokens.as_bytes(),
        &payroll_pda.key.to_bytes(),
    ];
    let (expected_payroll_index_pda, index_bump) = Pubkey::find_program_address(payroll_index_pda_account_seeds, program_id);
    if expected_payroll_index_pda != *payroll_index_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    if payroll_token_pda.data_len() <= 0 {
        total_reward_tokens += 1;
        let payroll_token_pda_lamports = Rent::get()?.minimum_balance(PAYROLL_TOKEN_PDA_LEN);
        let payroll_token_pda_signer_seeds :&[&[u8]; 5] = &[
            PAYROLL_TOKEN_SEED,
            parsed_current_payroll_index.as_bytes(),
            &reward_token_mint_account.key.to_bytes(),
            &payroll_pda.key.to_bytes(),
            &[token_bump],
        ];
        let create_pda_token_account_ix = system_instruction::create_account(
            &account.key,
            &payroll_token_pda.key,
            payroll_token_pda_lamports,
            PAYROLL_TOKEN_PDA_LEN.try_into().unwrap(),
            &program_id,
        );
        invoke_signed(
            &create_pda_token_account_ix,
            &[
                account.clone(),
                payroll_token_pda.clone(),
                system_program_account.clone(),
            ],
            &[payroll_token_pda_signer_seeds],
        )?;
        let payroll_token_data = PayrollToken {
            account_type: POOL_PAYROLL_TOKEN_ACCOUNT_TYPE,
            reward_token_mint_account: *reward_token_mint_account.key,
            reward_withdrawn_amount: 0,
            total_reward_amount: amount,
            payroll_pda: *payroll_pda.key,
            creator: *account.key,
        };
        payroll_token_data.serialize(&mut &mut payroll_token_pda.data.borrow_mut()[..])?;
        let payroll_index_pda_lamports = Rent::get()?.minimum_balance(PAYROLL_INDEX_PDA_LEN);
        let payroll_index_pda_signer_seeds :&[&[u8]; 4] = &[
            PAYROLL_INDEX_SEED,
            parsed_number_of_reward_tokens.as_bytes(),
            &payroll_pda.key.to_bytes(),
            &[index_bump],
        ];
        let create_pda_index_account_ix = system_instruction::create_account(
            &account.key,
            &payroll_index_pda.key,
            payroll_index_pda_lamports,
            PAYROLL_INDEX_PDA_LEN.try_into().unwrap(),
            &program_id,
        );
        
        invoke_signed(
            &create_pda_index_account_ix,
            &[
                account.clone(),
                payroll_index_pda.clone(),
                system_program_account.clone(),
            ],
            &[payroll_index_pda_signer_seeds],
        )?;
        let payroll_index_data = PayrollIndex {
            account_type: POOL_PAYROLL_INDEX_ACCOUNT_TYPE,
            reward_token_mint_account: *reward_token_mint_account.key,
            index: total_reward_tokens + 1,
            payroll_pda: *payroll_pda.key,
            creator: *account.key,
        };
        payroll_index_data.serialize(&mut &mut payroll_index_pda.data.borrow_mut()[..])?;
    } else {
        let mut payroll_token_data = PayrollToken::try_from_slice(&payroll_token_pda.data.borrow())?;
        payroll_token_data.total_reward_amount += amount;
        payroll_token_data.serialize(&mut &mut payroll_token_pda.data.borrow_mut()[..])?;
    }

    if reward_token_mint_account.key != system_program_account.key {
        let ix = spl_token::instruction::transfer(
            &token_program_account.key,
            &reward_token_source_associated_account.key,
            &reward_token_dest_associated_account.key,
            &account.key,
            &[],
            amount,
        )?;
        // let signers_seeds: &[&[u8]; 1] = &[
        //     &pda_account.key.to_bytes(),
        // ];
        invoke(
            &ix,
            &[
                reward_token_source_associated_account.clone(),
                reward_token_dest_associated_account.clone(),
                account.clone(),
                token_program_account.clone(),
            ],
        )?;
    } else {
        let sol_ix = system_instruction::transfer(
            account.key,
            payroll_pda.key,
            amount,
        );
        invoke(
            &sol_ix,
            &[
                account.clone(),
                payroll_pda.clone(),
            ],
        )?;
    }

    Ok(())
}
