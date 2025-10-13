use crate::fuzz_accounts::FuzzAccounts;
use crate::types::*;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ")]
#[discriminator([4u8, 144u8, 132u8, 71u8, 116u8, 23u8, 151u8, 80u8])]
pub struct ClaimRewardsInstruction {
    pub accounts: ClaimRewardsInstructionAccounts,
    pub data: ClaimRewardsInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(ClaimRewardsInstructionData)]
#[storage(FuzzAccounts)]
pub struct ClaimRewardsInstructionAccounts {
    #[account(mut)]
    pub bucket: TridentAccount,

    pub contribution_record: TridentAccount,

    #[account(mut)]
    pub pool_contribution: TridentAccount,

    #[account(mut)]
    pub trading_pool: TridentAccount,

    #[account(mut)]
    pub creator_profile: TridentAccount,

    #[account(mut)]
    pub contributor_token_account: TridentAccount,

    #[account(mut)]
    pub vault_token_account: TridentAccount,

    #[account(mut)]
    pub program_state: TridentAccount,

    #[account(mut)]
    pub fee_vault: TridentAccount,

    #[account(mut, signer)]
    pub contributor: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,

    #[account(address = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")]
    pub associated_token_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ClaimRewardsInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for ClaimRewardsInstruction {
    type IxAccounts = FuzzAccounts;
}
