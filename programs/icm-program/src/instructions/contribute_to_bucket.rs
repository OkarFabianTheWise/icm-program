use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, Transfer, transfer},
};
use crate::state::{Bucket, ContributionRecord, BucketStatus};
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(bucket_name: String, token_mint: Pubkey)]
pub struct ContributeToBucket<'info> {
    #[account(
        mut,
        seeds = [b"bucket", bucket_name.as_bytes(), bucket.creator.as_ref()],
        bump = bucket.bump
    )]
    pub bucket: Account<'info, Bucket>,
    #[account(
        init_if_needed,
        payer = contributor,
        // contribution_record: discriminator (8) + contributor (32) + bucket (32) + token_mint (32) + amount (8) + timestamp (8) = 120
        space = 8 + 32 + 32 + 32 + 8 + 8,
        seeds = [b"contribution", bucket.key().as_ref(), contributor.key().as_ref(), token_mint.key().as_ref()],
        bump
    )]
    pub contribution_record: Account<'info, ContributionRecord>,
    #[account(
        init_if_needed,
        payer = contributor,
        // pool_contribution: discriminator (8) + pool_id (32) + contributor (32) + contribution_amount (8) + pool_share_percentage (8) + claimed (1) = 89
        space = 8 + 32 + 32 + 8 + 8 + 1,
        seeds = [b"pool_contribution", bucket.key().as_ref(), contributor.key().as_ref(), token_mint.key().as_ref()],
        bump
    )]
    pub pool_contribution: Account<'info, crate::state::PoolContribution>,
    #[account(mut)]
    pub contributor_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = bucket,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub contributor: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn contribute_to_bucket_handler(
    ctx: Context<ContributeToBucket>,
    bucket_name: String,
    token_mint: Pubkey,
    amount: u64,
) -> Result<()> {
    let bucket = &mut ctx.accounts.bucket;
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

    // Transfer tokens to vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.contributor_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.contributor.to_account_info(),
        },
    );
    transfer(transfer_ctx, amount)?;

    // Update or create contribution record
    let contribution_record = &mut ctx.accounts.contribution_record;
    if contribution_record.contributor == Pubkey::default() {
        // New contribution
        contribution_record.contributor = ctx.accounts.contributor.key();
        contribution_record.bucket = bucket.key();
        contribution_record.token_mint = token_mint;
        contribution_record.amount = amount;
        contribution_record.timestamp = clock.unix_timestamp;
        // Increment contributor_count for new contributor
        bucket.contributor_count = bucket.contributor_count.checked_add(1).ok_or(ErrorCode::Overflow)?;
    } else {
        // Add to existing contribution
        contribution_record.amount = contribution_record
            .amount
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;
        contribution_record.timestamp = clock.unix_timestamp; // update timestamp if desired
    }

    // Update raised_amount (or total_contributions if that's the field used)
    bucket.raised_amount = bucket
        .raised_amount
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;

    let pool_contribution = &mut ctx.accounts.pool_contribution;
    pool_contribution.pool_id = bucket.key();
    pool_contribution.contributor = ctx.accounts.contributor.key();
    pool_contribution.contribution_amount = pool_contribution
        .contribution_amount
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;
    pool_contribution.claimed = false;

    msg!(
        "Contribution of {} tokens to bucket '{}' by {}",
        amount,
        bucket.name,
        ctx.accounts.contributor.key()
    );
    Ok(())
}
