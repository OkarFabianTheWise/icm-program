use crate::fuzz_accounts::FuzzAccounts;
use crate::types::*;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ")]
#[discriminator([201u8, 226u8, 234u8, 16u8, 70u8, 155u8, 131u8, 206u8])]
pub struct SwapTokensInstruction {
    pub accounts: SwapTokensInstructionAccounts,
    pub data: SwapTokensInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(SwapTokensInstructionData)]
#[storage(FuzzAccounts)]
pub struct SwapTokensInstructionAccounts {
    #[account(mut)]
    pub trade_record: TridentAccount,

    #[account(mut, signer)]
    pub creator: TridentAccount,

    #[account(mut)]
    pub bucket: TridentAccount,

    pub input_mint: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub input_mint_program: TridentAccount,

    pub output_mint: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub output_mint_program: TridentAccount,

    pub vault_input_token_account: TridentAccount,

    #[account(mut)]
    pub vault_output_token_account: TridentAccount,

    pub raydium_amm_program: TridentAccount,

    #[account(mut)]
    pub amm: TridentAccount,

    pub amm_authority: TridentAccount,

    #[account(mut)]
    pub pool_coin_token_account: TridentAccount,

    #[account(mut)]
    pub pool_pc_token_account: TridentAccount,

    #[account(mut)]
    pub user_source_token_account: TridentAccount,

    #[account(mut)]
    pub user_destination_token_account: TridentAccount,

    #[account(signer)]
    pub user_authority: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,

    #[account(address = "SysvarRent111111111111111111111111111111111")]
    pub rent: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct SwapTokensInstructionData {
    pub in_amount: u64,

    pub quoted_out_amount: u64,

    pub slippage_bps: u16,
}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for SwapTokensInstruction {
    type IxAccounts = FuzzAccounts;
}
