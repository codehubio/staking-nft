#![allow(clippy::integer_arithmetic)]
use {
    thiserror::Error,
    solana_program::program_error::ProgramError,
};
/// Reasons the program may fail
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ContractError {
  // 0
  #[error("Not a system account")]
  NotASystemAccount,
  // 1
  #[error("Invalid depositor")]
  InvalidDepositor,
  // 2
  #[error("Fund already withdrawn")]
  FundAlreadyWithdrawn,
  // 3
  #[error("Reward already withdrawn")]
  RewardAlreadyWithdrawn,
  // 4
  #[error("Invalid deposit token")]
  InvalidDepositToken,
  // 5
  #[error("Invalid time range")]
  InvalidTimeRange,
  // 6
  #[error("Invalid PDA account")]
  InvalidPdaAccount,
  // 7
  #[error("Invalid ATA account")]
  InvalidAtaAccount,
  // 8
  #[error("Invalid deposited amount")]
  InvalidDepositAmount,
  // 9
  #[error("Invalid pool creator")]
  InvalidPoolCreator,
  // a
  #[error("Invalid reward token")]
  InvalidRewardToken,
  // b
  #[error("No reward for this payroll")]
  NoRewardPayroll,
  // c
  #[error("Invalid withdrawn address")]
  InvalidWithdrawnAddress,
  // d
  #[error("Invalid program account")]
  InvalidProgramAccount,
  // e
  #[error("Transfer error")]
  TransferError,
  // f
  #[error("Invalid collection")]
  InvalidCollection,
  
}

impl From<ContractError> for ProgramError {
  fn from(e: ContractError) -> Self {
    ProgramError::Custom(e as u32)
  }
}