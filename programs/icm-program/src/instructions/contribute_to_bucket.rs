use crate::error::ErrorCode;
use crate::state::{Bucket, BucketStatus, ContributionRecord, PoolContribution, ProgramState};
use crate::constants::usdc_id;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(bucket_name: String)]
pub struct ContributeToBucket<'info> {
    #[account(
        mut,
        seeds = [b"bucket", bucket_name.as_bytes(), bucket.creator.as_ref()],
        bump = bucket.bump
    )]
    pub bucket: Box<Account<'info, Bucket>>,

    ///### Dangerous init_if_needed
    #[account(
        init_if_needed,
        payer = contributor,
        // contribution_record: discriminator (8) + contributor (32) + bucket (32) + token_mint (32) + amount (8) + timestamp (8) = 120
        space = 8 + ContributionRecord::INIT_SPACE,
        seeds = [b"contribution", bucket.key().as_ref(), contributor.key().as_ref(), usdc_mint.key().as_ref()],
        bump
    )]
    pub contribution_record: Box<Account<'info, ContributionRecord>>,

    ///### Dangerous init_if_needed
    #[account(
        init_if_needed,
        payer = contributor,
        // pool_contribution: discriminator (8) + pool_id (32) + contributor (32) + contribution_amount (8) + pool_share_percentage (8) + claimed (1) = 89
        space = 8 + PoolContribution::INIT_SPACE,
        seeds = [b"pool_contribution", bucket.key().as_ref(), contributor.key().as_ref(), usdc_mint.key().as_ref()],
        bump
    )]
    pub pool_contribution: Box<Account<'info, PoolContribution>>,

    #[account(mut)]
    pub contributor_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = bucket,
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    ///## @audit put the address constraint to ensure the usdc mint is the specified usdc mint in the constants file
    #[account(address = usdc_id())]
    pub usdc_mint: Box<Account<'info, Mint>>,
    
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
    pub contributor: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn contribute_to_bucket_handler(
    ctx: Context<ContributeToBucket>,
    _bucket_name: String,
    amount: u64,
) -> Result<()> {
    let bucket = &mut ctx.accounts.bucket;
    let program_state = &mut ctx.accounts.program_state;
    let clock = Clock::get()?;

    // Validations
    require!(
        bucket.status == BucketStatus::Raising,
        ErrorCode::BucketNotRaising
    );
    require!(
        clock.unix_timestamp < bucket.contribution_deadline,
        ErrorCode::ContributionDeadlinePassed
    );
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(program_state.initialized, ErrorCode::ProgramNotInitialized);

    // usdc_mint is validated by account constraint `address = usdc_id()`

    // Calculate fee amount (0.5% = 50 basis points)
    let fee_amount = (amount as u128)
        .checked_mul(program_state.fee_rate_bps as u128)
        .ok_or(ErrorCode::Overflow)?
        .checked_div(10000)
        .ok_or(ErrorCode::Overflow)? as u64;

    let net_amount = amount
        .checked_sub(fee_amount)
        .ok_or(ErrorCode::InsufficientFunds)?;

    // fee transfer
    //@audit: this if statement can be removed.
    // What this means is if the fee amount is not greater than zero, it will skip the fee sending
    // and fee is never zero
    if fee_amount > 0 {
        let fee_transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.contributor_token_account.to_account_info(),
                to: ctx.accounts.fee_vault.to_account_info(),
                authority: ctx.accounts.contributor.to_account_info(),
            },
        );
        transfer(fee_transfer_ctx, fee_amount)?;

        // Update total fees collected
        program_state.total_fees_collected = program_state
            .total_fees_collected
            .checked_add(fee_amount)
            .ok_or(ErrorCode::Overflow)?;
    }

    // remaining amount after fee is collected
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.contributor_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.contributor.to_account_info(),
        },
    );
    transfer(transfer_ctx, net_amount)?;

    // Update or create contribution record (using net amount for bucket tracking)
    let contribution_record = &mut ctx.accounts.contribution_record;
    if contribution_record.contributor == Pubkey::default() {
        // New contribution
        contribution_record.contributor = ctx.accounts.contributor.key();
        contribution_record.bucket = bucket.key();
        contribution_record.token_mint = ctx.accounts.usdc_mint.key();
        contribution_record.amount = net_amount;
        contribution_record.timestamp = clock.unix_timestamp;
        // Increment contributor_count for new contributor
        bucket.contributor_count = bucket
            .contributor_count
            .checked_add(1)
            .ok_or(ErrorCode::Overflow)?;
    } else {
        // Add to existing contribution
        contribution_record.amount = contribution_record
            .amount
            .checked_add(net_amount)
            .ok_or(ErrorCode::Overflow)?;
        contribution_record.timestamp = clock.unix_timestamp; // update timestamp if desired
    }

    // Update raised_amount with net amount (after fees)
    bucket.raised_amount = bucket
        .raised_amount
        .checked_add(net_amount)
        .ok_or(ErrorCode::Overflow)?;

    let pool_contribution = &mut ctx.accounts.pool_contribution;

    // Initialize or update pool contribution
    if pool_contribution.pool_id == Pubkey::default() {
        // New pool contribution
        pool_contribution.pool_id = bucket.key();
        pool_contribution.contributor = ctx.accounts.contributor.key();
        pool_contribution.contribution_amount = net_amount;
        pool_contribution.pool_share_percentage = 0; // Will be calculated later when trading starts
        pool_contribution.claimed = false;
    } else {
        // Add to existing pool contribution
        pool_contribution.contribution_amount = pool_contribution
            .contribution_amount
            .checked_add(net_amount)
            .ok_or(ErrorCode::Overflow)?;
    }

    msg!(
        "Contribution of {} USDC tokens ({} net after {} fee) to bucket '{}' by {}",
        amount,
        net_amount,
        fee_amount,
        bucket.name,
        ctx.accounts.contributor.key()
    );
    Ok(())
}
