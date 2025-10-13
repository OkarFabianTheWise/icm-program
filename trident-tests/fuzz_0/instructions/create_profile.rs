use crate::fuzz_accounts::FuzzAccounts;
use crate::types::*;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ")]
#[discriminator([225u8, 205u8, 234u8, 143u8, 17u8, 186u8, 50u8, 220u8])]
pub struct CreateProfileInstruction {
    pub accounts: CreateProfileInstructionAccounts,
    pub data: CreateProfileInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(CreateProfileInstructionData)]
#[storage(FuzzAccounts)]
pub struct CreateProfileInstructionAccounts {
    #[account(mut)]
    pub creator_profile: TridentAccount,

    #[account(mut, signer)]
    pub creator: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct CreateProfileInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for CreateProfileInstruction {
    type IxAccounts = FuzzAccounts;
}
