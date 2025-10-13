use crate::fuzz_accounts::FuzzAccounts;
use crate::types::*;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ")]
#[discriminator([198u8, 212u8, 171u8, 109u8, 144u8, 215u8, 174u8, 89u8])]
pub struct WithdrawFeesInstruction {
    pub accounts: WithdrawFeesInstructionAccounts,
    pub data: WithdrawFeesInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(WithdrawFeesInstructionData)]
#[storage(FuzzAccounts)]
pub struct WithdrawFeesInstructionAccounts {
    #[account(mut)]
    pub program_state: TridentAccount,

    #[account(mut)]
    pub fee_vault: TridentAccount,

    #[account(mut)]
    pub owner_token_account: TridentAccount,

    #[account(signer)]
    pub owner: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct WithdrawFeesInstructionData {
    pub amount: Option<u64>,
}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for WithdrawFeesInstruction {
    type IxAccounts = FuzzAccounts;
}
