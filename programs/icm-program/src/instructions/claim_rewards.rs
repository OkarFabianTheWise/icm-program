use crate::error::ErrorCode;
use crate::state::{Bucket, BucketStatus, ContributionRecord, ProgramState};
use crate::utils::close_bucket_util;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token::{self, TokenAccount, Transfer};

#[derive(Accounts)]
#[instruction(bucket_name: String, token_mint: Pubkey)]
pub struct ClaimRewards<'info> {
    #[account(
        //@audit , bucket was not declared with mutable status and is used in code as
        // borrowed mutable reference in programs/icm-program/src/utils.rs::close_bucket_util()
        // bucket is used as a mutable reference in the code, put the mut for best practices to 
        // changes are made onchain
        mut,
        seeds = [b"bucket", bucket_name.as_bytes(), bucket.creator.as_ref()],
        bump = bucket.bump
    )]
    pub bucket: Account<'info, Bucket>,

    #[account(
        seeds = [b"contribution", bucket.key().as_ref(), contributor.key().as_ref(), token_mint.as_ref()],
        bump
    )]
    pub contribution_record: Account<'info, ContributionRecord>,

    #[account(
        mut,
        seeds = [b"pool_contribution", bucket.key().as_ref(), contributor.key().as_ref(), token_mint.as_ref()],
        bump
    )]
    pub pool_contribution: Account<'info, crate::state::PoolContribution>,

    #[account(
        mut,
        seeds = [b"trading_pool", bucket.name.as_bytes(), bucket.creator.as_ref()],
        bump
    )]
    pub trading_pool: Account<'info, crate::state::TradingPool>,

    #[account(
        mut,
        seeds = [b"creator_profile", bucket.creator.as_ref()],
        bump
    )]
    pub creator_profile: Account<'info, crate::state::CreatorProfile>,

    #[account(mut)]
    pub contributor_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"program_state"],
        bump = program_state.bump
    )]
    pub program_state: Account<'info, ProgramState>,

    #[account(
        mut,
        associated_token::mint = program_state.usdc_mint,
        associated_token::authority = program_state,
    )]
    pub fee_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub contributor: Signer<'info>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn claim_rewards_handler(ctx: Context<ClaimRewards>) -> Result<()> {
    let bucket = &mut ctx.accounts.bucket;
    let contribution_record = &ctx.accounts.contribution_record;
    let clock = Clock::get()?;

    // Check if trading is elapsed but not closed, and user is authorized
    if bucket.status == BucketStatus::Trading && clock.unix_timestamp > bucket.trading_deadline {
        require!(
            contribution_record.contributor == ctx.accounts.contributor.key(),
            ErrorCode::UnauthorizedContributor
        );
        close_bucket_util(
            bucket,
            &mut ctx.accounts.trading_pool,
            &mut ctx.accounts.creator_profile,
            clock.unix_timestamp,
        );
        msg!(
            "Bucket closed by contributor {} after trading elapsed",
            ctx.accounts.contributor.key()
        );
    }

    // Validations
    require!(
        bucket.status == BucketStatus::Closed,
        ErrorCode::BucketNotClosed
    );
    require!(
        contribution_record.contributor == ctx.accounts.contributor.key(),
        ErrorCode::UnauthorizedContributor
    );

    // Calculate proportional share
    let vault_balance = ctx.accounts.vault_token_account.amount;
    let user_share = (contribution_record.amount as u128)
        .checked_mul(vault_balance as u128)
        .unwrap()
        .checked_div(bucket.raised_amount as u128)
        .unwrap() as u64;

    // Calculate creator fee if this is creator claiming
    let amount_to_transfer = if contribution_record.contributor == bucket.creator {
        let creator_fee = (vault_balance as u128)
            .checked_mul(bucket.creator_fee_percent as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;
        user_share
            .checked_add(creator_fee)
            .ok_or(ErrorCode::Overflow)?
    } else {
        user_share
    };

    require!(amount_to_transfer > 0, ErrorCode::NoRewardsAvailable);

    // Calculate fee on rewards (0.5% from program state)
    let program_state = &mut ctx.accounts.program_state;
    let fee_amount = (amount_to_transfer as u128)
        .checked_mul(program_state.fee_rate_bps as u128)
        .ok_or(ErrorCode::Overflow)?
        .checked_div(10000)
        .ok_or(ErrorCode::Overflow)? as u64;

    let net_amount = amount_to_transfer
        .checked_sub(fee_amount)
        .ok_or(ErrorCode::InsufficientFunds)?;

    let bucket_seeds = &[
        b"bucket",
        bucket.name.as_bytes(),
        bucket.creator.as_ref(),
        &[bucket.bump],
    ];
    let bucket_signer = &[&bucket_seeds[..]];

    // Transfer fee to program fee vault if there's a fee
    if fee_amount > 0 {
        let fee_transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.fee_vault.to_account_info(),
                authority: bucket.to_account_info(),
            },
            bucket_signer,
        );
        token::transfer(fee_transfer_ctx, fee_amount)?;

        // Update total fees collected
        program_state.total_fees_collected = program_state
            .total_fees_collected
            .checked_add(fee_amount)
            .ok_or(ErrorCode::Overflow)?;
    }

    // Transfer net amount to contributor
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.contributor_token_account.to_account_info(),
            authority: bucket.to_account_info(),
        },
        bucket_signer,
    );
    token::transfer(transfer_ctx, net_amount)?;

    // Persist PoolContribution account
    let pool_contribution = &mut ctx.accounts.pool_contribution;
    pool_contribution.claimed = true;

    msg!(
        "Rewards claimed: {} tokens ({} net after {} fee) by {}",
        amount_to_transfer,
        net_amount,
        fee_amount,
        ctx.accounts.contributor.key()
    );
    Ok(())
}
