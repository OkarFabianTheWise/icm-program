use crate::state::ProgramState;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + ProgramState::INIT_SPACE,
        seeds = [b"program_state"],
        bump
    )]
    pub program_state: Box<Account<'info, ProgramState>>,

    #[account(
        init,
        payer = owner,
        associated_token::mint = usdc_mint,
        associated_token::authority = program_state,
    )]
    pub fee_vault: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_program_handler(
    ctx: Context<InitializeProgram>,
    fee_rate_bps: u16,
) -> Result<()> {
    let program_state = &mut ctx.accounts.program_state;
    let clock = Clock::get()?;

    // Validate fee rate (max 10% = 1000 bps)
    require!(fee_rate_bps <= 1000, crate::error::ErrorCode::FeeTooHigh);

    program_state.owner = ctx.accounts.owner.key();
    program_state.fee_rate_bps = fee_rate_bps;
    program_state.usdc_mint = ctx.accounts.usdc_mint.key();
    program_state.total_fees_collected = 0;
    program_state.initialized = true;
    program_state.created_at = clock.unix_timestamp;
    program_state.bump = ctx.bumps.program_state;

    msg!("Program initialized with fee rate: {} bps", fee_rate_bps);

    Ok(())
}
