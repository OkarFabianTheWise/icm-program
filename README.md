### 🧱 Anchor Program Structure

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

declare_id!("YourProgramIDHere111111111111111111111111111111");

#[program]
pub mod etf_trading_pool {
    use super::*;

    pub fn create_bucket(
        ctx: Context<CreateBucket>,
        token_mints: Vec<Pubkey>,
        contribution_duration: i64,
        trading_duration: i64,
        creator_fee_bps: u16,
        name: String,
    ) -> Result<()> {
        let bucket = &mut ctx.accounts.bucket;

        require!(token_mints.len() >= 2, ErrorCode::InsufficientTokens);
        require!(creator_fee_bps <= 2000, ErrorCode::ExcessiveFee); // 20% = 2000 bps
        require!(trading_duration <= MAX_TRADING_DURATION, ErrorCode::InvalidTradingDuration);

        let clock = Clock::get()?;
        bucket.creator = ctx.accounts.creator.key();
        bucket.token_mints = token_mints;
        bucket.created_at = clock.unix_timestamp;
        bucket.contribution_deadline = clock.unix_timestamp + contribution_duration;
        bucket.trading_deadline = bucket.contribution_deadline + trading_duration;
        bucket.creator_fee_bps = creator_fee_bps;
        bucket.name = name;
        bucket.status = BucketStatus::Raising;

        Ok(())
    }

    // TODO: implement contribute_to_bucket
    // TODO: implement swap_tokens (restricted to creator after contribution phase)
    // TODO: implement close_bucket_and_withdraw
    // TODO: implement claim_share (proportional withdrawal for contributors)

}
```

---

### 🏗️ State Definitions

```rust
#[account]
pub struct Bucket {
    pub creator: Pubkey,
    pub token_mints: Vec<Pubkey>,
    pub created_at: i64,
    pub contribution_deadline: i64,
    pub trading_deadline: i64,
    pub creator_fee_bps: u16,
    pub name: String,
    pub status: BucketStatus,
    pub total_contributions: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BucketStatus {
    Raising,
    Trading,
    Closed,
}

#[account]
pub struct Contribution {
    pub user: Pubkey,
    pub bucket: Pubkey,
    pub amount: u64,
    pub mint: Pubkey,
    pub bump: u8,
}
```

---

### 📦 Accounts Context

```rust
#[derive(Accounts)]
#[instruction(token_mints: Vec<Pubkey>, name: String)]
pub struct CreateBucket<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [b"bucket", creator.key().as_ref(), name.as_bytes()],
        bump,
        space = 8 + 32 + (32 * token_mints.len()) + 8*3 + 2 + 4 + name.len() + 1 + 8 + 1,
    )]
    pub bucket: Account<'info, Bucket>,

    pub system_program: Program<'info, System>,
}
```

---

### ⚙️ Constants

```rust
const MAX_TRADING_DURATION: i64 = 6 * 30 * 24 * 60 * 60; // 6 months in seconds
```

---

### ✅ Next TODO Blocks

* [ ] `contribute_to_bucket(ctx, bucket_id, mint, amount)`

  * User deposits token
  * Creates `Contribution` account
  * Adds to `bucket.total_contributions`

* [ ] `swap_tokens(ctx, bucket_id, from_token, to_token)`

  * Restricted to creator
  * Only allowed after contribution\_deadline
  * Only swaps within `bucket.token_mints`

* [ ] `close_bucket_and_withdraw(ctx, bucket_id)`

  * Closes trading
  * Calculates final balances in vault
  * Updates `BucketStatus::Closed`

* [ ] `claim_share(ctx, bucket_id)`

  * Contributor withdraws based on their share
  * Creator gets fees + their share

* [ ] PDAs for:

  * Vault Token Accounts (one per token\_mint per bucket)
  * Contribution accounts (per user per token per bucket)
