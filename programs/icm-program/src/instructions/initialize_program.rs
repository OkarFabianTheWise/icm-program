use crate::state::ProgramState;
use crate::error::ErrorCode;

// import the usdc mint and verify the mint is the specified mainnet mint address
use crate::constants::usdc_id;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + ProgramState::INIT_SPACE,
        // make the seeds more deterministic and use extra inputs like the crate_id since only the program can call this fucntion
        seeds = [b"program_state", crate::ID.to_bytes().as_ref()],
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

    //@ audit make sure the program is not initialized and once it is initialized, it cannot be initialized again
    require!(program_state.initialized == false, ErrorCode::ProgramInitialized);
   
    let clock = Clock::get()?;

    // Validate fee rate (max 10% = 1000 bps)
    require!(fee_rate_bps <= 1000, crate::error::ErrorCode::FeeTooHigh);

    //@ audit verify owner check here, not sure what owner to put so i wont put a check here yet
    program_state.owner = ctx.accounts.owner.key();
    program_state.fee_rate_bps = fee_rate_bps;
    
    // @ audit verify the usdc mint is the exact mint passed in the constants folder
    program_state.usdc_mint = ctx.accounts.usdc_mint.key();
    require!(program_state.usdc_mint == usdc_id(), ErrorCode::InvalidMint);

    program_state.total_fees_collected = 0;
    program_state.initialized = true;
    program_state.created_at = clock.unix_timestamp;
    program_state.bump = ctx.bumps.program_state;
    msg!("Program initialized with fee rate: {} bps", fee_rate_bps);

    Ok(())
}
