use anchor_lang::prelude::*;
use crate::state::{Bucket, BucketStatus};
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(bucket_name: String)]
pub struct CloseBucket<'info> {
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
    #[account(
        mut,
        seeds = [b"creator_profile", creator.key().as_ref()],
        bump
    )]
    pub creator_profile: Account<'info, crate::state::CreatorProfile>,
    #[account(mut)]
    pub creator: Signer<'info>,
}

pub fn close_bucket_handler(ctx: Context<CloseBucket>) -> Result<()> {
    let bucket = &mut ctx.accounts.bucket;
    let clock = Clock::get()?;

    // Validations
    require!(
        bucket.creator == ctx.accounts.creator.key(),
        ErrorCode::UnauthorizedCreator
    );
    require!(
        bucket.status == BucketStatus::Trading,
        ErrorCode::BucketNotTrading
    );
    require!(
        clock.unix_timestamp >= bucket.trading_deadline,
        ErrorCode::TradingStillActive
    );

    bucket.status = BucketStatus::Closed;
    bucket.closed_at = clock.unix_timestamp;

    // Persist TradingPool account
    let trading_pool = &mut ctx.accounts.trading_pool;
    trading_pool.phase = crate::state::PoolPhase::Closed;
    trading_pool.trading_end_time = Some(clock.unix_timestamp);

    // Persist CreatorProfile account
    let creator_profile = &mut ctx.accounts.creator_profile;
    creator_profile.successful_pools += 1;
    creator_profile.total_volume_managed += bucket.raised_amount;

    msg!("Bucket '{}' closed for claims", bucket.name);
    Ok(())
}
