// state.rs
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum PoolPhase {
    Raising,
    Trading,
    Closed,
}

#[account]
#[derive(InitSpace)]
pub struct Bucket {
    pub creator: Pubkey,
    #[max_len(50)]
    pub name: String,
    #[max_len(3)]
    pub token_mints: Vec<Pubkey>,
    pub contribution_deadline: i64,
    pub trading_deadline: i64,
    pub creator_fee_percent: u16, // Basis points (e.g., 500 = 5%)
    pub status: BucketStatus,
    pub trading_started_at: i64,
    ///### This was hard coded to 0 on initilalization
    ///### in the initialize_bucket()
    pub closed_at: i64,
    pub bump: u8,
    pub creator_profile: Pubkey,
    // @ make all fee of type u64 consistently
    pub performance_fee: u64,
    pub raised_amount: u64,
    pub contributor_count: u32,
}

#[account]
#[derive(InitSpace)]
pub struct ContributionRecord {
    pub contributor: Pubkey,
    pub bucket: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum BucketStatus {
    Raising,
    Trading,
    Closed,
}

impl Bucket {
    pub fn is_trading_open(&self) -> bool {
        self.status == BucketStatus::Trading
    }
}

#[account]
#[derive(InitSpace)]
pub struct TradingPool {
    pub pool_id: Pubkey,
    pub pool_bump: u8,
    pub creator: Pubkey,
    ///### @check Does the order matter in implementation?
    #[max_len(3)]
    pub token_bucket: Vec<Pubkey>,
    pub target_amount: u64,
    pub min_contribution: u64,
    pub max_contribution: u64,
    pub trading_duration: u64,
    pub created_at: i64,
    ///### @check why is the fundraising deadline == bucket.contribution_deadline and what impact would the fundraising deadline have on the codebase
    pub fundraising_deadline: i64,
    ///### @check why is this option<i64> and hardcoded to NOne on initialization
    pub trading_start_time: Option<i64>,
    ///### @check why is this option<i64> and hardcoded to NOne on initialization
    pub trading_end_time: Option<i64>,
    pub phase: PoolPhase,
    pub management_fee: u64,
}

#[account]
#[derive(InitSpace)]
pub struct PoolContribution {
    pub pool_id: Pubkey,
    pub contributor: Pubkey,
    pub contribution_amount: u64,
    pub pool_share_percentage: u64,
    pub claimed: bool,
}

#[account]
#[derive(InitSpace)]
pub struct TradeRecord {
    pub pool_id: Pubkey,
    pub trade_id: u64,
    pub timestamp: i64,
    pub trade_type: TradeType,
    pub from_token: Pubkey,
    pub to_token: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub success: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum TradeType {
    BuyToken,
    SellToken,
    Rebalance,
}

#[account]
#[derive(InitSpace)]
pub struct CreatorProfile {
    pub creator: Pubkey,
    pub pools_created: u32,
    pub successful_pools: u32,
    pub total_volume_managed: u64,
    pub reputation_score: u32,
    pub created_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct ProgramState {
    pub owner: Pubkey,
    pub fee_rate_bps: u16, // Fee rate in basis points (50 = 0.5%)
    pub usdc_mint: Pubkey,
    pub total_fees_collected: u64,
    ///## Since the program state is intialized once we keep a state for the program to ensure it can be initialized only once
    pub initialized: bool,
    pub created_at: i64,
    pub bump: u8,
}

