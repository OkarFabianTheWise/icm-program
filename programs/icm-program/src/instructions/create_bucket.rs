use crate::state::{Bucket, TradingPool, CreatorProfile, ProgramState};
use crate::constants::{usdc_id, BUCKET_CREATION_FEE};
use crate::error::ErrorCode;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

mod initialization;
mod validation;

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateBucket<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Bucket::INIT_SPACE,
        seeds = [b"bucket", name.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub bucket: Box<Account<'info, Bucket>>,

    #[account(
        init,
        payer = creator,
        // calculated space = 478
        space = 8 + TradingPool::INIT_SPACE,
        seeds = [b"trading_pool", name.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub trading_pool: Box<Account<'info, crate::state::TradingPool>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = usdc_mint,
        associated_token::authority = bucket,
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    //@ audit: Ensure creator profile exists before allowing bucket creation
    #[account(
        seeds = [b"creator_profile", creator.key().as_ref()],
        bump
    )]
    pub creator_profile: Account<'info, CreatorProfile>,

    #[account(
        mut,
        seeds = [b"program_state"],
        bump = program_state.bump
    )]
    pub program_state: Box<Account<'info, ProgramState>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = program_state,
    )]
    pub fee_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub creator_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = usdc_id())]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn create_bucket_handler(
    ctx: Context<CreateBucket>,
    name: String,
    mut token_mints: Vec<Pubkey>,
    contribution_window_minutes: u32,
    trading_window_minutes: u32,
    creator_fee_percent: u16,
    target_amount: u64,
    min_contribution: u64,
    max_contribution: u64,
    management_fee: u64,
) -> Result<()> {
    let program_state = &mut ctx.accounts.program_state;
    
    // Validate program is initialized
    require!(program_state.initialized, ErrorCode::ProgramNotInitialized);
    
    msg!("validating inputs");
    validation::validate_inputs(
        &name,
        &token_mints,
        contribution_window_minutes,
        trading_window_minutes,
        creator_fee_percent,
    )?;
    msg!("inputs validated");

    // Collect bucket creation fee (0.7 USDC)
    msg!("collecting bucket creation fee: {} micro USDC", BUCKET_CREATION_FEE);
    let fee_transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.creator_token_account.to_account_info(),
            to: ctx.accounts.fee_vault.to_account_info(),
            authority: ctx.accounts.creator.to_account_info(),
        },
    );
    transfer(fee_transfer_ctx, BUCKET_CREATION_FEE)?;

    // Update total fees collected
    program_state.total_fees_collected = program_state
        .total_fees_collected
        .checked_add(BUCKET_CREATION_FEE)
        .ok_or(ErrorCode::Overflow)?;
    msg!("bucket creation fee collected successfully");

    let clock = Clock::get()?;
    msg!("initializing bucket");
    initialization::initialize_bucket(
        &mut ctx.accounts.bucket,
        &name,
        &mut token_mints,
        contribution_window_minutes,
        trading_window_minutes,
        creator_fee_percent,
        management_fee,
        &ctx.accounts.creator,
        &ctx.accounts.creator_profile,
        clock.unix_timestamp,
        ctx.bumps.bucket,
    )?;
    msg!("bucket initialized");

    msg!("initializing trading pool");
    initialization::initialize_trading_pool(
        &mut ctx.accounts.trading_pool,
        &ctx.accounts.bucket,
        &ctx.accounts.creator,
        &ctx.accounts.bucket.token_mints,
        target_amount,
        min_contribution,
        max_contribution,
        trading_window_minutes,
        management_fee,
        clock.unix_timestamp,
        ctx.bumps.trading_pool,
    )?;
    msg!("trading pool initialized");

    msg!("bucket{:?} with name {:?} and trading pool {:?} created successfully", ctx.accounts.bucket.key(), ctx.accounts.bucket.name, ctx.accounts.trading_pool.key());
    Ok(())
}
