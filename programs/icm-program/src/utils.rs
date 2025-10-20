use crate::state::{Bucket, BucketStatus, CreatorProfile, PoolPhase, TradingPool};
use crate::error::ErrorCode;
use anchor_lang::prelude::*;

/// Utility function to close a bucket and update related accounts
/// Returns Result for proper error handling
pub fn close_bucket_util(
    bucket: &mut Account<Bucket>,
    trading_pool: &mut Account<TradingPool>,
    creator_profile: &mut Account<CreatorProfile>,
    now: i64,
) -> Result<()> {
    msg!("Bucket close util start");
    bucket.status = BucketStatus::Closed;
    bucket.closed_at = now;
    trading_pool.phase = PoolPhase::Closed;
    trading_pool.trading_end_time = Some(now);
    
    // Update creator profile with proper error handling
    creator_profile.successful_pools = creator_profile
        .successful_pools
        .checked_add(1)
        .ok_or(ErrorCode::Overflow)?;
    creator_profile.total_volume_managed = creator_profile
        .total_volume_managed
        .checked_add(bucket.raised_amount)
        .ok_or(ErrorCode::Overflow)?;
    
    msg!("bucket close util ended");
    Ok(())
}
