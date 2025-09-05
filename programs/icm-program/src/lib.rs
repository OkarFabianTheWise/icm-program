#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod error;
pub mod constants;

pub use instructions::*;
pub use state::*;
pub use error::*;
pub use constants::*;

declare_id!("BXwnU4VFAoQ3ES4SXZnXdMGsDufRrWwyrmXUNLhCH9JT");

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
        contribution_window_days: u32,
        trading_window_days: u32,
        creator_fee_percent: u16,
        target_amount: u64,
        min_contribution: u64,
        max_contribution: u64,
        management_fee: u16,
    ) -> Result<()> {
        create_bucket::create_bucket_handler(
            ctx,
            name,
            token_mints,
            contribution_window_days,
            trading_window_days,
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
        token_mint: Pubkey,
        amount: u64,
    ) -> Result<()> {
        contribute_to_bucket::contribute_to_bucket_handler(ctx, bucket_name, token_mint, amount)
    }

    pub fn start_trading(ctx: Context<StartTrading>, bucket_name: String) -> Result<()> {
        start_trading::start_trading_handler(ctx, bucket_name)
    }

    pub fn close_bucket(ctx: Context<CloseBucket>) -> Result<()> {
        close_bucket::close_bucket_handler(ctx)
    }

    pub fn claim_rewards(
        ctx: Context<ClaimRewards>,
        token_mint: Pubkey,
    ) -> Result<()> {
        claim_rewards::claim_rewards_handler(ctx, token_mint)
    }
    
    pub fn swap_tokens(
        ctx: Context<SwapTokens>,
        route_plan: Vec<u8>,
        in_amount: u64,
        quoted_out_amount: u64,
        slippage_bps: u16,
        platform_fee_bps: u16,
    ) -> Result<()> {
        swap_tokens::swap_tokens_handler(ctx, route_plan, in_amount, quoted_out_amount, slippage_bps, platform_fee_bps)
    }
}


