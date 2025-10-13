use anchor_lang::prelude::*;
use crate::state::{Bucket, BucketStatus};

#[inline(never)]
pub fn initialize_bucket(
    bucket: &mut Account<Bucket>,
    name: &String,
    token_mints: &mut Vec<Pubkey>,
    contribution_window_minutes: u32,
    trading_window_minutes: u32,
    creator_fee_percent: u16,
    management_fee: u16,
    creator: &Signer,
    now: i64,
    bump: u8,
) -> Result<()> {
    bucket.creator = creator.key();
    bucket.name = name.clone();
    bucket.token_mints = token_mints.clone();
    bucket.contribution_deadline = now + (contribution_window_minutes as i64 * 60);
    bucket.trading_deadline = bucket.contribution_deadline + (trading_window_minutes as i64 * 60);
    bucket.creator_fee_percent = creator_fee_percent;
    bucket.status = BucketStatus::Raising;
    bucket.trading_started_at = 0;
    bucket.closed_at = 0;
    bucket.bump = bump;
    
    // Derive the creator profile PDA
    let (creator_profile_pda, _) = Pubkey::find_program_address(
        &[b"creator_profile", creator.key().as_ref()],
        &crate::ID
    );
    bucket.creator_profile = creator_profile_pda;
    bucket.performance_fee = management_fee;
    Ok(())
}

#[inline(never)]
pub fn initialize_trading_pool(
    trading_pool: &mut Account<crate::state::TradingPool>,
    bucket: &Account<Bucket>,
    creator: &Signer,
    token_bucket: &Vec<Pubkey>,
    target_amount: u64,
    min_contribution: u64,
    max_contribution: u64,
    trading_window_minutes: u32,
    management_fee: u16,
    now: i64,
    bump: u8,
) -> Result<()> {
    trading_pool.pool_id = bucket.key();
    trading_pool.pool_bump = bump;
    trading_pool.creator = creator.key();
    trading_pool.token_bucket = token_bucket.clone();
    trading_pool.target_amount = target_amount;
    trading_pool.min_contribution = min_contribution;
    trading_pool.max_contribution = max_contribution;
    trading_pool.trading_duration = trading_window_minutes as u64 * 60;
    trading_pool.created_at = now;
    trading_pool.fundraising_deadline = bucket.contribution_deadline;
    trading_pool.trading_start_time = None;
    trading_pool.trading_end_time = None;
    trading_pool.phase = crate::state::PoolPhase::Raising;
    trading_pool.management_fee = management_fee;
    Ok(())
}
