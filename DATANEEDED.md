## ✅ On-Chain: Scalable & Efficient to Save in Anchor

These are lightweight, essential state records that scale well on Solana:

```rust
// 1. Core Pool Entity
pub struct TradingPool {
    pub pool_id: Pubkey,
    pub pool_bump: u8,
    pub creator: Pubkey,
    pub strategy_description: String,
    pub token_bucket: Vec<Pubkey>,
    pub target_amount: u64,
    pub min_contribution: u64,
    pub max_contribution: u64,
    pub trading_duration: u64,
    pub created_at: i64,
    pub fundraising_deadline: i64,
    pub trading_start_time: Option<i64>,
    pub trading_end_time: Option<i64>,
    pub phase: PoolPhase,
    pub raised_amount: u64,
    pub contributor_count: u32,
    pub management_fee: u16,
    pub performance_fee: u16,
    pub trade_count: u32,
    pub last_trade_time: Option<i64>,
}

// 2. Contributions
pub struct PoolContribution {
    pub pool_id: Pubkey,
    pub contributor: Pubkey,
    pub contribution_amount: u64,
    pub contribution_timestamp: i64,
    pub pool_share_percentage: u64,
    pub claimed: bool,
}

// 3. Trade Records (individual trades)
pub struct TradeRecord {
    pub pool_id: Pubkey,
    pub trade_id: u64,
    pub timestamp: i64,
    pub trade_type: TradeType,
    pub from_token: Pubkey,
    pub to_token: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub slippage: u16,
    pub success: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum TradeType {
    BuyToken,
    SellToken,
    Rebalance,
}

// 4. Creator reputation minimal tracking
pub struct CreatorProfile {
    pub creator: Pubkey,
    pub pools_created: u32,
    pub successful_pools: u32,
    pub total_volume_managed: u64,
    pub reputation_score: u32,
    pub created_at: i64,
}
```


## ⚠️ Off-Chain: Better Handled Externally

These items are large, frequently updated, or analytical—better off-loaded to off-chain systems:

```rust
// 5. Portfolio Snapshots (heavy data, use DB)
pub struct PortfolioSnapshot {
    pub pool_id: Pubkey,
    pub timestamp: i64,
    pub total_value_usdc: u64,
    pub token_balances: Vec<TokenBalance>,
    pub pnl_percentage: i16,
}

pub struct TokenBalance {
    pub token_mint: Pubkey,
    pub amount: u64,
    pub value_usdc: u64,
}

// 6. Pool Performance Analytics
pub struct PoolPerformance {
    pub pool_id: Pubkey,
    pub current_pnl: i16,
    pub peak_pnl: i16,
    pub drawdown: i16,
    pub total_trades: u32,
    pub successful_trades: u32,
    pub last_updated: i64,
    pub roi_annualized: i16,
}

// 7. User Profile Aggregates
pub struct UserProfile {
    pub user: Pubkey,
    pub total_pools_joined: u32,
    pub active_contributions: Vec<Pubkey>,
    pub completed_contributions: Vec<Pubkey>,
    pub total_contributed: u64,
    pub total_pnl: i64,
}
```