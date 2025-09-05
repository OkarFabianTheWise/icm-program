use anchor_lang::prelude::*;
use crate::state::{Bucket, BucketStatus};
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(bucket_name: String)]
pub struct StartTrading<'info> {
    #[account(
        mut,
        seeds = [b"bucket", bucket_name.as_bytes(), creator.key().as_ref()],
        bump = bucket.bump
    )]
    pub bucket: Account<'info, Bucket>,
    #[account(
        mut,
        seeds = [b"trading_pool", bucket_name.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub trading_pool: Account<'info, crate::state::TradingPool>,
    #[account(mut)]
    pub creator: Signer<'info>,
}

pub fn start_trading_handler(
    ctx: Context<StartTrading>,
    bucket_name: String,
) -> Result<()> {
    let bucket = &mut ctx.accounts.bucket;
    let clock = Clock::get()?;

    // Validations
    require!(
        bucket.creator == ctx.accounts.creator.key(),
        ErrorCode::UnauthorizedCreator
    );
    require!(
        bucket.status == BucketStatus::Raising,
        ErrorCode::BucketNotRaising
    );
    require!(
        clock.unix_timestamp >= bucket.contribution_deadline,
        ErrorCode::ContributionStillActive
    );
    require!(bucket.raised_amount > 0, ErrorCode::NoContributions);

    // calculate 30% of the money and lend it
    // keep track for retrieval when the trading phase is finished
    bucket.status = BucketStatus::Trading;
    bucket.trading_started_at = clock.unix_timestamp;

    // Update the real TradingPool account
    let trading_pool = &mut ctx.accounts.trading_pool;
    trading_pool.phase = crate::state::PoolPhase::Trading;
    trading_pool.trading_start_time = Some(clock.unix_timestamp);

    msg!("Trading started for bucket '{}'", bucket.name);
    Ok(())
}
