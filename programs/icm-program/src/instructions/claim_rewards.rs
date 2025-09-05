use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{Bucket, ContributionRecord, BucketStatus};
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(bucket_name: String, token_mint: Pubkey)]
pub struct ClaimRewards<'info> {
    #[account(
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
    #[account(mut)]
    pub contributor_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub contributor: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn claim_rewards_handler(
    ctx: Context<ClaimRewards>,
    token_mint: Pubkey,
) -> Result<()> {
    let bucket = &ctx.accounts.bucket;
    let contribution_record = &ctx.accounts.contribution_record;

    // Validations
    require!(
        bucket.status == BucketStatus::Closed,
        ErrorCode::BucketNotClosed
    );
    require!(
        contribution_record.contributor == ctx.accounts.contributor.key(),
        ErrorCode::UnauthorizedContributor
    );
    require!(
        bucket.token_mints.contains(&token_mint),
        ErrorCode::TokenNotAllowed
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
        user_share + creator_fee
    } else {
        user_share
    };

    require!(amount_to_transfer > 0, ErrorCode::NoRewardsAvailable);

    // Transfer tokens from vault to contributor
    let seeds = &[
        b"bucket",
        bucket.name.as_bytes(),
        bucket.creator.as_ref(),
        &[bucket.bump],
    ];
    let signer = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.contributor_token_account.to_account_info(),
            authority: bucket.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, amount_to_transfer)?;

    // Persist PoolContribution account
    let pool_contribution = &mut ctx.accounts.pool_contribution;
    pool_contribution.claimed = true;

    msg!(
        "Rewards claimed: {} tokens by {}",
        amount_to_transfer,
        ctx.accounts.contributor.key()
    );
    Ok(())
}
