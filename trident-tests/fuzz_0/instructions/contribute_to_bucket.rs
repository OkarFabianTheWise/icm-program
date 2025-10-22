use crate::fuzz_accounts::FuzzAccounts;
use crate::types::*;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ")]
#[discriminator([155u8, 45u8, 223u8, 38u8, 100u8, 65u8, 26u8, 133u8])]
pub struct ContributeToBucketInstruction {
    pub accounts: ContributeToBucketInstructionAccounts,
    pub data: ContributeToBucketInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(ContributeToBucketInstructionData)]
#[storage(FuzzAccounts)]
pub struct ContributeToBucketInstructionAccounts {
    #[account(mut)]
    pub bucket: TridentAccount,

    #[account(mut)]
    pub contribution_record: TridentAccount,

    #[account(mut)]
    pub pool_contribution: TridentAccount,

    #[account(mut)]
    pub contributor_token_account: TridentAccount,

    #[account(mut)]
    pub vault_token_account: TridentAccount,

    #[account(address = "2RgRJx3z426TMCL84ZMXTRVCS5ee7iGVE4ogqcUAd3tg")]
    pub usdc_mint: TridentAccount,

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

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ContributeToBucketInstructionData {
    pub bucket_name: String,

    pub amount: u64,
}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for ContributeToBucketInstruction {
    type IxAccounts = FuzzAccounts;
}
