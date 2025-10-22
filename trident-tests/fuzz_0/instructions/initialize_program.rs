use crate::fuzz_accounts::FuzzAccounts;
use crate::types::*;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ")]
#[discriminator([176u8, 107u8, 205u8, 168u8, 24u8, 157u8, 175u8, 103u8])]
pub struct InitializeProgramInstruction {
    pub accounts: InitializeProgramInstructionAccounts,
    pub data: InitializeProgramInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(InitializeProgramInstructionData)]
#[storage(FuzzAccounts)]
pub struct InitializeProgramInstructionAccounts {
    #[account(mut)]
    pub program_state: TridentAccount,

    #[account(mut)]
    pub fee_vault: TridentAccount,

    #[account(address = "2RgRJx3z426TMCL84ZMXTRVCS5ee7iGVE4ogqcUAd3tg")]
    pub usdc_mint: TridentAccount,

    #[account(mut, signer, address = "11111111111111111111111111111112")]
    pub owner: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,

    #[account(address = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")]
    pub associated_token_program: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct InitializeProgramInstructionData {
    pub fee_rate_bps: u16,
}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for InitializeProgramInstruction {
    type IxAccounts = FuzzAccounts;
}
