use crate::common::{
    get_current_payroll_index,
    get_or_create_payroll_by_index, verify_ata_account, verify_program_account,
    verify_system_account, DECIMAL_REWARD, STAKING_PAYROLL_ACCOUNT_TYPE,
};
use crate::error::ContractError;
use crate::schemas::states::payroll::Payroll;
/// Define the type of state stored in accounts
use crate::schemas::states::pool::Pool;
use crate::schemas::states::staking_account::StakingAccount;
use crate::schemas::states::staking_payroll::{
    StakingPayroll, STAKING_PAYROLL_PDA_LEN, STAKING_PAYROLL_SEED,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_associated_token_account::instruction as spl_instruction;
use std::convert::TryInto;

use crate::schemas::instructions::reward_redemption::RewardRedemption;

// Program entrypoint's implementation
pub fn process_instruction<'a>(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let pool_pda_account = next_account_info(accounts_iter)?;
    let dst_account = next_account_info(accounts_iter)?;
    let staking_payroll_account = next_account_info(accounts_iter)?;
    let reward_pda = next_account_info(accounts_iter)?;
    let reward_token_mint = next_account_info(accounts_iter)?;
    let reward_token_pool_associated_account = next_account_info(accounts_iter)?;
    let reward_token_dest_associated_account = next_account_info(accounts_iter)?;
    let payroll_pda = next_account_info(accounts_iter)?;
    let token_program_account = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;
    // check for account
    if payroll_pda.data_len() <= 0 {
        return Err(ContractError::NoRewardPayroll.into());
    }
    let inst_data = RewardRedemption::try_from_slice(instruction_data)?;
    let mut payroll_data = Payroll::try_from_slice(&payroll_pda.data.borrow())?;
    let mut staking_account = StakingAccount::try_from_slice(&pda_account.data.borrow())?;
    verify_system_account(&account)?;
    verify_program_account(pool_pda_account, program_id)?;
    verify_program_account(pda_account, program_id)?;
    let clock = Clock::get()?;
    let pool_data = Pool::try_from_slice(&pool_pda_account.data.borrow())?;
    // only check if dao is not system program
    verify_ata_account(
        &pool_pda_account.key,
        reward_token_pool_associated_account.key,
        &pool_data.reward_token_mint_address,
    )?;
    verify_ata_account(
        &staking_account.withdrawn_address,
        reward_token_dest_associated_account.key,
        &pool_data.reward_token_mint_address,
    )?;
    if staking_account.withdrawn_address != *dst_account.key {
        return Err(ContractError::InvalidWithdrawnAddress.into());
    }
    if pool_data.reward_token_mint_address != *reward_token_mint.key {
        return Err(ContractError::InvalidRewardToken.into());
    }
    let index = inst_data.index;
    if staking_account.first_payroll_index > index {
        return Err(ContractError::InvalidTimeRange.into());
    }
    let (expected_payroll, _payroll_index) = match get_or_create_payroll_by_index(
        index,
        program_id,
        account,
        pool_pda_account,
        payroll_pda,
        system_program_account,
    ) {
        Ok(p) => p,
        Err(_err) => return Err(ContractError::InvalidTimeRange.into()),
    };
    if expected_payroll != *payroll_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }

    let rewarder_pda_account_seeds: &[&[u8]; 2] =
        &[&payroll_pda.key.to_bytes(), &pool_pda_account.key.to_bytes()];
    let (expected_rewarder, reward_bump) =
        Pubkey::find_program_address(rewarder_pda_account_seeds, program_id);
    if expected_rewarder != *reward_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }
    let rewarder_pda_signer_seeds: &[&[u8]; 3] = &[
        &payroll_pda.key.to_bytes(),
        &pool_pda_account.key.to_bytes(),
        &[reward_bump],
    ];
    let ata_account_data_len = reward_token_dest_associated_account.data_len();
    // msg!("ata dst address: {:?}, {:?}" ,staking_account.withdrawn_address, dst_account.key);
    if ata_account_data_len <= 0 {
        let create_token_account_ix = spl_instruction::create_associated_token_account(
            &account.key,
            &staking_account.withdrawn_address,
            &pool_data.reward_token_mint_address,
            &token_program_account.key,
        );
        invoke(
            &create_token_account_ix,
            &[
                account.clone(),
                reward_token_dest_associated_account.clone(),
                dst_account.clone(),
                reward_token_mint.clone(),
                system_program_account.clone(),
                token_program_account.clone(),
            ],
            // &[signers_seeds]
        )?;
    }

    // fund withdrawn
    let now = clock.unix_timestamp as u64;
    if staking_account.withdrawn_at > 0 {
        let latest_payroll_index = get_current_payroll_index(
            staking_account.withdrawn_at,
            pool_data.reward_period,
            pool_data.start_at,
        );
        if latest_payroll_index - 1 < payroll_data.index {
            return Err(ContractError::InvalidPdaAccount.into());
        }
    }
    let parsed_index = payroll_data.index.to_string();
    let staking_payroll_account_seeds: &[&[u8]; 4] = &[
        STAKING_PAYROLL_SEED,
        parsed_index.as_bytes(),
        &pool_pda_account.key.to_bytes(),
        &pda_account.key.to_bytes(),
    ];
    let (staking_payroll_pda, staking_payroll_bump) =
        Pubkey::find_program_address(&staking_payroll_account_seeds[..], program_id);
    if staking_payroll_pda != *staking_payroll_account.key {
        return Err(ContractError::RewardAlreadyWithdrawn.into());
    }
    // already withdrawn
    let mut total_withdrawn_reward = 0;
    let staking_payroll_data;
    if staking_payroll_account.data_len() > 0 {
        // return Err(ContractError::RewardAlreadyWithdrawn.into());
        staking_payroll_data =
            StakingPayroll::try_from_slice(&staking_payroll_account.data.borrow())?;
        total_withdrawn_reward = staking_payroll_data.reward_withdrawn_amount;
    }
    let staking_payroll_signers_seeds: &[&[u8]; 5] = &[
        STAKING_PAYROLL_SEED,
        parsed_index.as_bytes(),
        &pool_pda_account.key.to_bytes(),
        &pda_account.key.to_bytes(),
        &[staking_payroll_bump],
    ];
    let lamports_required = Rent::get()?.minimum_balance(STAKING_PAYROLL_PDA_LEN);
    let create_staking_payroll_pda_account_ix = system_instruction::create_account(
        &account.key,
        &staking_payroll_account.key,
        lamports_required,
        STAKING_PAYROLL_PDA_LEN.try_into().unwrap(),
        &program_id,
    );
    invoke_signed(
        &create_staking_payroll_pda_account_ix,
        &[
            account.clone(),
            staking_payroll_account.clone(),
            system_program_account.clone(),
        ],
        &[staking_payroll_signers_seeds],
    )?;

    if now < payroll_data.claimable_after {
        return Err(ContractError::InvalidTimeRange.into());
    }
    let reward_amount = std::cmp::max(
        staking_account.deposited_power * payroll_data.rate_reward
            / u64::pow(10, DECIMAL_REWARD)
            - total_withdrawn_reward,
        0,
    );
    if reward_amount == 0 {
        return Err(ContractError::RewardAlreadyWithdrawn.into());
    }
    payroll_data.reward_withdrawn_amount += reward_amount;
    payroll_data.serialize(&mut &mut payroll_pda.data.borrow_mut()[..])?;
    // tranfer the interest
    let ix = spl_token::instruction::transfer(
        &token_program_account.key,
        &reward_token_pool_associated_account.key,
        &reward_token_dest_associated_account.key,
        &reward_pda.key,
        &[],
        reward_amount,
    )?;

    // msg!("src: {:?}, dest: {:?}, pda: {:?}, token_program: {:?}", &reward_token_pool_associated_account.key, &reward_token_dest_associated_account.key, &pda_account.key, &token_program_account.key);
    invoke_signed(
        &ix,
        &[
            reward_token_pool_associated_account.clone(),
            reward_token_dest_associated_account.clone(),
            reward_pda.clone(),
            token_program_account.clone(),
        ],
        &[rewarder_pda_signer_seeds],
    )?;
    staking_account.withdrawn_reward_amount += reward_amount;
    // tranfer the interest
    staking_account.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    let updated_staking_payroll_data = StakingPayroll {
        account_type: STAKING_PAYROLL_ACCOUNT_TYPE,
        staking_pda_account: *pda_account.key,
        deposited_power: staking_account.deposited_power,
        total_pool_deposited_power: payroll_data.total_deposited_power,
        total_reward_amount: payroll_data.total_reward_amount,
        reward_withdrawn_amount: reward_amount,
        index: payroll_data.index,
        withdrawn_at: now,
    };
    updated_staking_payroll_data
        .serialize(&mut &mut staking_payroll_account.data.borrow_mut()[..])?;

    Ok(())
}
