use crate::fuzz_accounts::FuzzAccounts;
use crate::types::*;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ")]
#[discriminator([26u8, 158u8, 9u8, 222u8, 245u8, 138u8, 207u8, 185u8])]
pub struct CreateBucketInstruction {
    pub accounts: CreateBucketInstructionAccounts,
    pub data: CreateBucketInstructionData,
}

/// Instruction Accounts
#[derive(Arbitrary, Debug, Clone, TridentAccounts, Default)]
#[instruction_data(CreateBucketInstructionData)]
#[storage(FuzzAccounts)]
pub struct CreateBucketInstructionAccounts {
    #[account(
        mut,
        seeds = [b"bucket", ]
    )]
    pub bucket: TridentAccount,

    #[account(mut)]
    pub trading_pool: TridentAccount,

    #[account(mut)]
    pub vault_token_account: TridentAccount,

    pub usdc_mint: TridentAccount,

    #[account(mut, signer)]
    pub creator: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,

    #[account(address = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")]
    pub associated_token_program: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct CreateBucketInstructionData {
    pub name: String,

    pub token_mints: Vec<TridentPubkey>,

    pub contribution_window_minutes: u32,

    pub trading_window_minutes: u32,

    pub creator_fee_percent: u16,

    pub target_amount: u64,

    pub min_contribution: u64,

    pub max_contribution: u64,

    pub management_fee: u16,
}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for CreateBucketInstruction {
    type IxAccounts = FuzzAccounts;

    fn set_data(&mut self, trident: &mut trident, fuzz_accounts: &mut Self::IxAccounts){
        self.data.input = trident.gen_range(0..=u8::MAX);
    }
}
