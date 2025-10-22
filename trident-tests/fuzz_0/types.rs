use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

/// File containing all custom types which can be used
/// in transactions and instructions or invariant checks.
///
/// You can define your own custom types here.

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct Bucket {
    pub creator: TridentPubkey,

    pub name: String,

    pub token_mints: Vec<TridentPubkey>,

    pub contribution_deadline: i64,

    pub trading_deadline: i64,

    pub creator_fee_percent: u16,

    pub status: BucketStatus,

    pub trading_started_at: i64,

    pub closed_at: i64,

    pub bump: u8,

    pub creator_profile: TridentPubkey,

    pub performance_fee: u64,

    pub raised_amount: u64,

    pub contributor_count: u32,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub enum BucketStatus {
    #[default]
    Raising,

    Trading,

    Closed,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ContributionRecord {
    pub contributor: TridentPubkey,

    pub bucket: TridentPubkey,

    pub token_mint: TridentPubkey,

    pub amount: u64,

    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct CreatorProfile {
    pub creator: TridentPubkey,

    pub pools_created: u32,

    pub successful_pools: u32,

    pub total_volume_managed: u64,

    pub reputation_score: u32,

    pub created_at: i64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct PoolContribution {
    pub pool_id: TridentPubkey,

    pub contributor: TridentPubkey,

    pub contribution_amount: u64,

    pub pool_share_percentage: u64,

    pub claimed: bool,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub enum PoolPhase {
    #[default]
    Raising,

    Trading,

    Closed,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ProgramState {
    pub owner: TridentPubkey,

    pub fee_rate_bps: u16,

    pub usdc_mint: TridentPubkey,

    pub total_fees_collected: u64,

    pub initialized: bool,

    pub created_at: i64,

    pub bump: u8,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct TradeRecord {
    pub pool_id: TridentPubkey,

    pub trade_id: u64,

    pub timestamp: i64,

    pub trade_type: TradeType,

    pub from_token: TridentPubkey,

    pub to_token: TridentPubkey,

    pub amount_in: u64,

    pub amount_out: u64,

    pub success: bool,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub enum TradeType {
    #[default]
    BuyToken,

    SellToken,

    Rebalance,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct TradingPool {
    pub pool_id: TridentPubkey,

    pub pool_bump: u8,

    pub creator: TridentPubkey,

    pub token_bucket: Vec<TridentPubkey>,

    pub target_amount: u64,

    pub min_contribution: u64,

    pub max_contribution: u64,

    pub trading_duration: u64,

    pub created_at: i64,

    pub fundraising_deadline: i64,

    pub trading_start_time: Option<i64>,

    pub trading_end_time: Option<i64>,

    pub phase: PoolPhase,

    pub management_fee: u64,
}
