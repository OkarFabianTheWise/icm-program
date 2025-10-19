use crate::state::ProgramState;
use crate::error::ErrorCode;

// import the usdc mint and verify the mint is the specified mainnet mint address
use crate::constants::{usdc_id, deployer_id};

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
        seeds = [b"program_state"/*, crate::ID.to_bytes().as_ref() */],
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

    //@ audit: Assert the usdc mint is equal to the passed in usdc mint address
    #[account(address = usdc_id() @ ErrorCode::InvalidMint)]
    pub usdc_mint: Box<Account<'info, Mint>>,

    //@ audit: Only the deployer can initialize the program
    #[account(
        mut,
        address = deployer_id() @ ErrorCode::UnauthorizedDeployer
    )]
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
    msg!("initializing program state");

    //@ audit make sure the program is not initialized and once it is initialized, it cannot be initialized again
    require!(program_state.initialized == false, ErrorCode::ProgramInitialized);
    msg!("program state init state is false");
   
    let clock = Clock::get()?;

    // Validate fee rate (max 20% = 2000 bps)
    // @audit, in the error enum, it was 20 percent but in code it is 10 percent, changed from 1000 to 2000 bps
    require!(fee_rate_bps <= 2000 , crate::error::ErrorCode::FeeTooHigh);
    require!(fee_rate_bps >=50, crate::error::ErrorCode::FeeIsTooLow);
    program_state.fee_rate_bps = fee_rate_bps;
    msg!("program state fee rate bps set");

    program_state.owner = ctx.accounts.owner.key();
    msg!("program state owner set to deployer");
    
    // @ audit verify the usdc mint is the exact mint passed in the constants folder
    program_state.usdc_mint = ctx.accounts.usdc_mint.key();
    // require!(program_state.usdc_mint == usdc_id(), ErrorCode::InvalidMint);
    msg!("usdc mint set");

    program_state.total_fees_collected = 0;
    program_state.initialized = true;
    program_state.created_at = clock.unix_timestamp;
    program_state.bump = ctx.bumps.program_state;
    msg!("Program initialized with fee rate: {} bps", fee_rate_bps);

    Ok(())
}
