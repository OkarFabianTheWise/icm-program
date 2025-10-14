use crate::state::ProgramState;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(
        mut,
        seeds = [b"program_state"],
        bump = program_state.bump,
        has_one = owner
    )]
    pub program_state: Box<Account<'info, ProgramState>>,

    #[account(
        mut,
        associated_token::mint = program_state.usdc_mint,
        associated_token::authority = program_state,
    )]
    pub fee_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub owner_token_account: Box<Account<'info, TokenAccount>>,

    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn withdraw_fees_handler(ctx: Context<WithdrawFees>, amount: Option<u64>) -> Result<()> {
    let program_state = &mut ctx.accounts.program_state;
    let fee_vault = &ctx.accounts.fee_vault;

    // Determine withdrawal amount (all available or specified amount)
    let withdraw_amount = match amount {
        Some(amt) => {
            require!(
                amt <= fee_vault.amount,
                crate::error::ErrorCode::InsufficientFunds
            );
            amt
        }
        None => fee_vault.amount,
    };

    require!(
        withdraw_amount > 0,
        crate::error::ErrorCode::InsufficientFunds
    );

    // Create seeds for program state PDA signing
    let seeds = &[b"program_state".as_ref(), &[program_state.bump]];
    let signer_seeds = &[&seeds[..]];

    // Transfer fees from vault to owner
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.fee_vault.to_account_info(),
            to: ctx.accounts.owner_token_account.to_account_info(),
            authority: program_state.to_account_info(),
        },
        signer_seeds,
    );

    token::transfer(transfer_ctx, withdraw_amount)?;

    msg!("Withdrew {} USDC in fees to owner", withdraw_amount);

    Ok(())
}
