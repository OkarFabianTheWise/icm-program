use crate::error::ErrorCode;
use crate::state::{Bucket, BucketStatus};
use crate::state::ProgramState;
use crate::utils::close_bucket_util;

use anchor_lang::prelude::*;
use anchor_spl::token::{close_account, CloseAccount, Token, TokenAccount};

#[derive(Accounts)]
// #[instruction(bucket_name: String)]
pub struct CloseBucket<'info> {
    ///@ audit: Make use of the close account macro for clean closing of the bucket account
    /// `reference = https://www.anchor-lang.com/docs/references/account-constraints#accountclose--target`
    #[account(
        mut,
        close = creator,
        seeds = [b"bucket", bucket.name.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub bucket: Box<Account<'info, Bucket>>,
    
    //@audit while closing the bucket, one also needs to close the bucket token_account as well
    #[account(
        mut,
        associated_token::mint = program_state.usdc_mint,
        associated_token::authority = bucket,
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    // @audit: Brought in program state to be able to properly declare what vault token account to be closed
    #[account()]
    pub program_state: Box<Account<'info,ProgramState>>, 

    #[account(
        mut,
        close = creator,
        seeds = [b"trading_pool", bucket.name.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub trading_pool: Account<'info, crate::state::TradingPool>,

    #[account(
        mut,
        seeds = [b"creator_profile", creator.key().as_ref()],
        bump
    )]
    pub creator_profile: Account<'info, crate::state::CreatorProfile>,

    #[account(mut)]
    pub creator: Signer<'info>,

    //@ audit: Token program used to close the token account
    pub token_program: Program<'info, Token>
}

pub fn close_bucket_handler(ctx: Context<CloseBucket>) -> Result<()> {
    let clock = Clock::get()?;
    msg!("closing bucket");

    // Do validations and CPI inside a limited scope to avoid borrow conflicts
    let bucket_name_for_log: String;
    {
        let bucket = &ctx.accounts.bucket;
        let trading_pool = &ctx.accounts.trading_pool;
        let creator = &ctx.accounts.creator;

        // Validations
        require!(
            bucket.creator == creator.key(),
            ErrorCode::UnauthorizedCreator
        );

        // @audit validate trading creator as well
        require!(
            trading_pool.creator == creator.key(),
            ErrorCode::UnauthorizedCreator
        );

        require!(
            bucket.status == BucketStatus::Trading,
            ErrorCode::BucketNotTrading
        );

        require!(
            clock.unix_timestamp >= bucket.trading_deadline,
            ErrorCode::TradingStillActive
        );

        // Close the vault token account before closing the bucket
        let name_bytes_owned = bucket.name.clone().into_bytes();
        let creator_key_bytes_owned = creator.key().to_bytes();
        let bump_bytes = [bucket.bump];
        let signer_seeds: &[&[u8]] = &[
            b"bucket",
            name_bytes_owned.as_slice(),
            creator_key_bytes_owned.as_ref(),
            &bump_bytes,
        ];
        let signer_arr = [signer_seeds];
        msg!("closing vault token account first");
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount{
                account: ctx.accounts.vault_token_account.to_account_info(),
                destination: ctx.accounts.creator.to_account_info(),
                authority: ctx.accounts.bucket.to_account_info()
            },
            &signer_arr,
        );
        close_account(cpi_ctx)?;

        bucket_name_for_log = bucket.name.clone();
    }

    msg!("vault token accounts closed");

    // Now take mutable borrows to update state after CPI
    let bucket_mut = &mut ctx.accounts.bucket;
    let trading_pool_mut = &mut ctx.accounts.trading_pool;
    let creator_profile_mut = &mut ctx.accounts.creator_profile;
    close_bucket_util(bucket_mut, trading_pool_mut, creator_profile_mut, clock.unix_timestamp)?;

    msg!("Bucket '{}' closed for claims", bucket_name_for_log);
    Ok(())
}
