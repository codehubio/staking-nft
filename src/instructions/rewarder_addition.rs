use crate::common::{
    get_or_create_payroll_by_index,
    recalculate_reward_rate, verify_ata_account, verify_system_account, POOL_PAYROLL_ACCOUNT_TYPE,
};
use crate::schemas::states::pool::Pool;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    pubkey::Pubkey,
};

use crate::schemas::states::payroll::Payroll;

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
    let payroll_pda = next_account_info(accounts_iter)?;
    let token_program_account = next_account_info(accounts_iter)?;
    let system_program_account = next_account_info(accounts_iter)?;

    verify_system_account(account)?;
    verify_ata_account(
        &account.key,
        reward_token_source_associated_account.key,
        &reward_token_mint_account.key,
    )?;
    let rewarder_pda_account_seeds: &[&[u8]; 2] =
        &[&payroll_pda.key.to_bytes(), &pool_pda_account.key.to_bytes()];
    let (expected_rewarder, _bump) =
        Pubkey::find_program_address(rewarder_pda_account_seeds, program_id);
    verify_ata_account(
        &expected_rewarder,
        reward_token_dest_associated_account.key,
        &reward_token_mint_account.key,
    )?;
    let inst_data = RewardAddition::try_from_slice(instruction_data)?;
    let current_payroll_index = inst_data.payroll_index;
    let updated_pool_data = Pool::try_from_slice(&pool_pda_account.data.borrow())?;
    let reward_period = updated_pool_data.reward_period;
    let start_at = updated_pool_data.start_at;
    let total_deposited_power = updated_pool_data.total_deposited_power;
    let match_token =
        updated_pool_data.reward_token_mint_address == *reward_token_mint_account.key;
    if match_token {
        return Err(ContractError::InvalidRewardToken.into());
    }
    let (current_payroll_pda, _currrent_payroll_index) = match get_or_create_payroll_by_index(
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
    if current_payroll_pda != *payroll_pda.key {
        return Err(ContractError::InvalidPdaAccount.into());
    }

    let amount = inst_data.amount;

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

    // update pool's reward info
    let mut payroll_total_reward: u64 = amount;
    let mut reward_withdrawn_amount = 0;
    if payroll_pda.data_len() > 0 {
        let current_payroll_data = Payroll::try_from_slice(&payroll_pda.data.borrow())?;
        payroll_total_reward += current_payroll_data.total_reward_amount;
        reward_withdrawn_amount = current_payroll_data.reward_withdrawn_amount;
    }
    // update pay roll reward's info
    let rate_reward = recalculate_reward_rate(
        total_deposited_power,
        payroll_total_reward,
    );
    let payroll_account_data = Payroll {
        account_type: POOL_PAYROLL_ACCOUNT_TYPE,
        total_deposited_power,
        total_reward_amount: payroll_total_reward,
        rate_reward,
        reward_withdrawn_amount,
        index: current_payroll_index,
        start_at,
        claimable_after: start_at + current_payroll_index * reward_period,
        pool_pda_account: *pool_pda_account.key,
        creator: *account.key,
    };
    payroll_account_data.serialize(&mut &mut payroll_pda.data.borrow_mut()[..])?;

    Ok(())
}
