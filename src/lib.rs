use solana_program::{
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    account_info::AccountInfo,
    pubkey::Pubkey,
};
pub mod instructions;
pub mod schemas;
pub mod common;
pub mod error;

entrypoint!(process_instruction);

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let (first, rest) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    match first {
        1 =>  instructions::pool_initialization::process_instruction(
                program_id,
                accounts,
                rest,
            ),
        2 =>  instructions::rewarder_addition::process_instruction(
            program_id,
            accounts,
            rest,
        ),
        3 =>  instructions::pool_deposit::process_instruction(
            program_id,
            accounts,
            rest,
        ),
        4 =>  instructions::reward_withdrawal::process_instruction(
            program_id,
            accounts,
            rest,
        ),
        5 =>  instructions::fund_withdrawal::process_instruction(
            program_id,
            accounts,
            rest,
        ),
        6 =>  instructions::token_data::process_instruction(
            program_id,
            accounts,
            rest,
        ),
        7 =>  instructions::collection_data::process_instruction(
            program_id,
            accounts,
            rest,
        ),
        _ => Err(ProgramError::InvalidInstructionData)
    }?;
    Ok(())
}
