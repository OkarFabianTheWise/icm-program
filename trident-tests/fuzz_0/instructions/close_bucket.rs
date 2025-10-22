use crate::fuzz_accounts::FuzzAccounts;
use crate::types::*;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ")]
#[discriminator([221u8, 162u8, 41u8, 191u8, 18u8, 157u8, 170u8, 20u8])]
pub struct CloseBucketInstruction {
    pub accounts: CloseBucketInstructionAccounts,
    pub data: CloseBucketInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(CloseBucketInstructionData)]
#[storage(FuzzAccounts)]
pub struct CloseBucketInstructionAccounts {
    #[account(mut)]
    pub bucket: TridentAccount,

    #[account(mut)]
    pub vault_token_account: TridentAccount,

    pub program_state: TridentAccount,

    #[account(mut)]
    pub trading_pool: TridentAccount,

    #[account(mut)]
    pub creator_profile: TridentAccount,

    #[account(mut, signer)]
    pub creator: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct CloseBucketInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for CloseBucketInstruction {
    type IxAccounts = FuzzAccounts;
}
