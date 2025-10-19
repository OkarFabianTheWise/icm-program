use crate::error::ErrorCode;
use crate::state::{Bucket, BucketStatus};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use solana_program::instruction::AccountMeta;
use solana_program::program::invoke;
// use solana_program::program_error::ProgramError;

//
// Inline copy of Solend's DepositReserveLiquidity instruction format
//
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct DepositReserveLiquidity {
    pub liquidity_amount: u64,
}

impl DepositReserveLiquidity {
    pub fn pack(&self) -> Vec<u8> {
        // Solend instructions use a 1-byte tag discriminator
        // DepositReserveLiquidity is instruction enum = 3
        // (see solend-program/src/instruction.rs)
        let mut data = Vec::with_capacity(9);
        data.push(3u8); // tag for DepositReserveLiquidity
        data.extend_from_slice(&self.liquidity_amount.to_le_bytes());
        data
    }
}

#[derive(Accounts)]
pub struct StartTrading<'info> {
    #[account(
        mut,
        // @audit: Since bucket name is unique and wont change, we can just reuse in the seeds
        seeds = [b"bucket", bucket.name.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub bucket: Account<'info, Bucket>,

    #[account(
        mut,
        seeds = [b"trading_pool", bucket.name.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub trading_pool: Account<'info, crate::state::TradingPool>,

    #[account(mut)]
    pub creator: Signer<'info>,

    // ---- Solend CPI accounts ----
    /// CHECK: Solend reserve account
    pub reserve: UncheckedAccount<'info>,
    /// CHECK: Reserve liquidity supply SPL Token account
    pub reserve_liquidity_supply: UncheckedAccount<'info>,
    /// CHECK: Lending market account
    pub lending_market: UncheckedAccount<'info>,
    /// CHECK: Lending market authority PDA
    pub lending_market_authority: UncheckedAccount<'info>,
    /// CHECK: Destination collateral SPL Token account (in our pool/vault)
    #[account(mut)]
    pub destination_collateral: UncheckedAccount<'info>,
    /// CHECK: Source liquidity SPL Token account (bucket funds)
    #[account(mut)]
    pub source_liquidity: UncheckedAccount<'info>,
    /// CHECK: Reserve collateral mint
    pub reserve_collateral_mint: UncheckedAccount<'info>,
    /// CHECK: Pyth oracle price account
    pub pyth_oracle: UncheckedAccount<'info>,
    /// CHECK: Switchboard oracle account
    pub switchboard_oracle: UncheckedAccount<'info>,

    /// CHECK: SPL Token program
    pub token_program: Program<'info, Token>,
    /// CHECK: Solend lending program on devnet
    /// Address: ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx
    pub solend_program: UncheckedAccount<'info>,
}

pub fn start_trading_handler(ctx: Context<StartTrading>, _bucket_name: String) -> Result<()> {
    let bucket = &mut ctx.accounts.bucket;
    let clock = Clock::get()?;

    // --- Validations ---
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

    // --- Calculate lend amount ---
    let lend_amount = bucket
        .raised_amount
        .checked_mul(30)
        .unwrap()
        .checked_div(100)
        .unwrap();

    // --- Build Solend CPI instruction ---
    let ix = solana_program::instruction::Instruction {
        program_id: ctx.accounts.solend_program.key(),
        accounts: vec![
            AccountMeta::new(ctx.accounts.source_liquidity.key(), false), // source liquidity
            AccountMeta::new(ctx.accounts.destination_collateral.key(), false), // destination collateral
            AccountMeta::new(ctx.accounts.reserve.key(), false),                // reserve
            AccountMeta::new(ctx.accounts.reserve_liquidity_supply.key(), false),
            AccountMeta::new(ctx.accounts.reserve_collateral_mint.key(), false),
            AccountMeta::new(ctx.accounts.lending_market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.lending_market_authority.key(), false),
            AccountMeta::new_readonly(ctx.accounts.pyth_oracle.key(), false),
            AccountMeta::new_readonly(ctx.accounts.switchboard_oracle.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.creator.key(), true), // liquidity authority
        ],
        data: DepositReserveLiquidity {
            liquidity_amount: lend_amount,
        }
        .pack(),
    };

    // --- Invoke CPI into Solend ---
    invoke(
        &ix,
        &[
            ctx.accounts.source_liquidity.to_account_info(),
            ctx.accounts.destination_collateral.to_account_info(),
            ctx.accounts.reserve.to_account_info(),
            ctx.accounts.reserve_liquidity_supply.to_account_info(),
            ctx.accounts.reserve_collateral_mint.to_account_info(),
            ctx.accounts.lending_market.to_account_info(),
            ctx.accounts.lending_market_authority.to_account_info(),
            ctx.accounts.pyth_oracle.to_account_info(),
            ctx.accounts.switchboard_oracle.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.creator.to_account_info(),
        ],
    )?;

    // --- Update local state ---
    bucket.status = BucketStatus::Trading;
    bucket.trading_started_at = clock.unix_timestamp;

    let trading_pool = &mut ctx.accounts.trading_pool;
    trading_pool.phase = crate::state::PoolPhase::Trading;
    trading_pool.trading_start_time = Some(clock.unix_timestamp);

    msg!(
        "Trading started: supplied {} to Solend reserve from bucket '{}'",
        lend_amount,
        bucket.name
    );
    Ok(())
}
