use crate::fuzz_accounts::FuzzAccounts;
use crate::types::*;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ")]
#[discriminator([92u8, 94u8, 243u8, 100u8, 155u8, 170u8, 217u8, 68u8])]
pub struct StartTradingInstruction {
    pub accounts: StartTradingInstructionAccounts,
    pub data: StartTradingInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(StartTradingInstructionData)]
#[storage(FuzzAccounts)]
pub struct StartTradingInstructionAccounts {
    #[account(mut)]
    pub bucket: TridentAccount,

    #[account(mut)]
    pub trading_pool: TridentAccount,

    #[account(mut, signer)]
    pub creator: TridentAccount,

    pub reserve: TridentAccount,

    pub reserve_liquidity_supply: TridentAccount,

    pub lending_market: TridentAccount,

    pub lending_market_authority: TridentAccount,

    #[account(mut)]
    pub destination_collateral: TridentAccount,

    #[account(mut)]
    pub source_liquidity: TridentAccount,

    pub reserve_collateral_mint: TridentAccount,

    pub pyth_oracle: TridentAccount,

    pub switchboard_oracle: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,

    pub solend_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct StartTradingInstructionData {
    pub bucket_name: String,
}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for StartTradingInstruction {
    type IxAccounts = FuzzAccounts;
}
