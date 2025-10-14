#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;
pub use utils::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("3B4MZ2emBqufVyzfCCcNrvjeJVE4JhnrAEyh5jsBpSaQ");

#[program]
mod icm_program {
    use super::*;
    pub fn create_profile(ctx: Context<CreateProfile>) -> Result<()> {
        instructions::create_profile::create_profile_handler(ctx)
    }

    pub fn create_bucket(
        ctx: Context<CreateBucket>,
        name: String,
        token_mints: Vec<Pubkey>,
        contribution_window_minutes: u32,
        trading_window_minutes: u32,
        creator_fee_percent: u16,
        target_amount: u64,
        min_contribution: u64,
        max_contribution: u64,
        management_fee: u64,
    ) -> Result<()> {
        create_bucket::create_bucket_handler(
            ctx,
            name,
            token_mints,
            contribution_window_minutes,
            trading_window_minutes,
            creator_fee_percent,
            target_amount,
            min_contribution,
            max_contribution,
            management_fee,
        )
    }

    pub fn contribute_to_bucket(
        ctx: Context<ContributeToBucket>,
        bucket_name: String,
        amount: u64,
    ) -> Result<()> {
        contribute_to_bucket::contribute_to_bucket_handler(ctx, bucket_name, amount)
    }

    pub fn start_trading(ctx: Context<StartTrading>, bucket_name: String) -> Result<()> {
        start_trading::start_trading_handler(ctx, bucket_name)
    }

    pub fn close_bucket(ctx: Context<CloseBucket>) -> Result<()> {
        close_bucket::close_bucket_handler(ctx)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        claim_rewards::claim_rewards_handler(ctx)
    }

    pub fn swap_tokens(
        ctx: Context<SwapTokens>,
        in_amount: u64,
        quoted_out_amount: u64,
        slippage_bps: u16,
    ) -> Result<()> {
        swap_tokens::swap_tokens_handler(ctx, in_amount, quoted_out_amount, slippage_bps)
    }

    pub fn initialize_program(ctx: Context<InitializeProgram>, fee_rate_bps: u16) -> Result<()> {
        initialize_program::initialize_program_handler(ctx, fee_rate_bps)
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>, amount: Option<u64>) -> Result<()> {
        withdraw_fees::withdraw_fees_handler(ctx, amount)
    }
}
