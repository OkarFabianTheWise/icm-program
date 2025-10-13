use anchor_lang::prelude::*;
use crate::state::{Bucket, BucketStatus, TradingPool, CreatorProfile, PoolPhase};

/// Utility function to close a bucket and update related accounts
pub fn close_bucket_util(
    bucket: &mut Account<Bucket>,
    trading_pool: &mut Account<TradingPool>,
    creator_profile: &mut Account<CreatorProfile>,
    now: i64,
) {
    bucket.status = BucketStatus::Closed;
    bucket.closed_at = now;
    trading_pool.phase = PoolPhase::Closed;
    trading_pool.trading_end_time = Some(now);
    creator_profile.successful_pools += 1;
    creator_profile.total_volume_managed += bucket.raised_amount;
}
