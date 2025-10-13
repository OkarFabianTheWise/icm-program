use anchor_lang::{prelude::*, solana_program::{instruction::Instruction, program::invoke_signed}};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::token::Token;
// anchor_lang::solana_program::instruction::AccountMeta;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::{constants::*, error::ErrorCode, state::*};

/// Borsh layout for Raydium CP-swap SwapBaseIn (minimal)
#[derive(BorshSerialize, BorshDeserialize)]
pub struct SwapBaseInData {
    pub amount_in: u64,
    pub minimum_out: u64,
}

/// Accounts required for Raydium CP-swap CPI.
///
/// Note: The `user_source_token_account`, `user_destination_token_account`, and `user_authority`
/// are actually the vault token accounts and vault authority PDA controlled by this program.
/// Raydium expects these fields to be named as such, but they are not user wallet accounts.
#[derive(Accounts)]
pub struct SwapTokens<'info> {
    #[account(
        init_if_needed,
        payer = creator,
        space = 8 + TradeRecord::INIT_SPACE,
        seeds = [b"trade_record", bucket.key().as_ref(), creator.key().as_ref()],
        bump
    )]
    pub trade_record: Account<'info, TradeRecord>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"bucket", bucket.name.as_bytes(), bucket.creator.as_ref()],
        bump,
    )]
    pub bucket: Account<'info, Bucket>,

    pub input_mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub input_mint_program: Program<'info, Token>,
    pub output_mint: InterfaceAccount<'info, Mint>,
    pub output_mint_program: Program<'info, Token>,
    #[account(
        seeds = [VAULT_SEED, bucket.to_account_info().key().as_ref(), input_mint.key().as_ref()],
        bump,
    )]
    pub vault_input_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [VAULT_SEED, bucket.to_account_info().key().as_ref(), output_mint.key().as_ref()],
        bump,
    )]
    pub vault_output_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Raydium AMM program: CP-swap on devnet/mainnet (set in your constants)
    /// CHECK: program
    pub raydium_amm_program: UncheckedAccount<'info>,

    // Minimal Raydium CP-swap required accounts (CHECK - follow Raydium exact ordering)
    /// CHECK: Raydium amm state account
    #[account(mut)]
    pub amm: UncheckedAccount<'info>,
    /// CHECK: Raydium amm authority PDA (readonly)
    pub amm_authority: UncheckedAccount<'info>,
    /// CHECK: pool coin token account (pool vault)
    #[account(mut)]
    pub pool_coin_token_account: UncheckedAccount<'info>,
    /// CHECK: pool pc token account (pool vault)
    #[account(mut)]
    pub pool_pc_token_account: UncheckedAccount<'info>,

    // These accounts are required by Raydium's CPI interface.
    // In this program, they are the vault token accounts and vault authority PDA (not user wallet accounts).
    #[account(mut)]
    /// Vault token account acting as the user source for Raydium swap (Raydium expects this field)
    pub user_source_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    /// Vault token account acting as the user destination for Raydium swap (Raydium expects this field)
    pub user_destination_token_account: InterfaceAccount<'info, TokenAccount>,
    /// Vault authority PDA acting as the user authority for Raydium swap (Raydium expects this field)
    pub user_authority: Signer<'info>,

    // token program used for transfers
    pub token_program: Program<'info, Token>,

    // rent sysvar may be required in the accounts list depending on Raydium build
    pub rent: Sysvar<'info, Rent>,
}

pub fn swap_tokens_handler(
    ctx: Context<SwapTokens>,
    in_amount: u64,
    quoted_out_amount: u64,       // Use this as `minimum_out` (slippage-protected)
    slippage_bps: u16,
) -> Result<()> {
    let bucket = &ctx.accounts.bucket;

    // === VALIDATION CHECKS (same as original) ===
    // 1. Creator authorization
    require!(
        bucket.creator == ctx.accounts.creator.key(),
        ErrorCode::UnauthorizedCreator
    );

    // 2. Trading phase validation
    require!(bucket.is_trading_open(), ErrorCode::TradingNotStarted);

    // 3. Token mint validation
    require!(
        bucket.token_mints.contains(&ctx.accounts.input_mint.key()),
        ErrorCode::InvalidTokenMint
    );
    require!(
        bucket.token_mints.contains(&ctx.accounts.output_mint.key()),
        ErrorCode::InvalidTokenMint
    );

    // 4. Timeline validation
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp > bucket.contribution_deadline,
        ErrorCode::TradingNotStarted
    );
    require!(
        clock.unix_timestamp <= bucket.trading_deadline,
        ErrorCode::TradingDeadlinePassed
    );

    // 5. Status validation
    require!(
        bucket.status == BucketStatus::Trading,
        ErrorCode::InvalidBucketStatus
    );

    // 6. Vault balance validation
    require!(
        ctx.accounts.vault_input_token_account.amount >= in_amount,
        ErrorCode::InsufficientVaultBalance
    );

    // 7. Input amount validation
    require!(in_amount > 0, ErrorCode::InvalidSwapAmount);

    // 8. Slippage validation (max 10% = 1000 bps)
    require!(slippage_bps <= 1000, ErrorCode::InvalidSwapAmount);

    // === BUILD Raydium SwapBaseIn CPI ===
    // Use quoted_out_amount as the minimum_out (client should compute this)
    let swap_data = SwapBaseInData {
        amount_in: in_amount,
        minimum_out: quoted_out_amount,
    }
    .try_to_vec()
    .map_err(|_| error!(ErrorCode::InvalidSwapAmount))?;

    // Build account metas in the exact order Raydium expects for CP-swap (minimal)
    // NOTE: If your pool requires additional accounts, add them in the correct order.
    let accounts = vec![
        // 0
        AccountMeta::new(ctx.accounts.amm.key(), false),
        // 1
        AccountMeta::new_readonly(ctx.accounts.amm_authority.key(), false),
        // 2
        AccountMeta::new(ctx.accounts.pool_coin_token_account.key(), false),
        // 3
        AccountMeta::new(ctx.accounts.pool_pc_token_account.key(), false),
        // 4 user source
        AccountMeta::new(ctx.accounts.user_source_token_account.key(), false),
        // 5 user destination
        AccountMeta::new(ctx.accounts.user_destination_token_account.key(), false),
        // 6 user authority (must sign)
        AccountMeta::new_readonly(ctx.accounts.user_authority.key(), true),
        // 7 token program
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        // 8 rent (some builds expect this)
        AccountMeta::new_readonly(ctx.accounts.rent.key(), false),
    ];

    let ix = Instruction {
        program_id: match ctx.accounts.raydium_amm_program.key().clone() {
            // prefer explicit constant if you have it; fallback to account passed in
            key => key,
        },
        accounts,
        data: swap_data,
    };

    // === signer seeds ===
    // vault authority is a PDA that signed for the vault_input_token_account.
    // We assume your vault ATA is a PDA derived with VAULT_SEED, bucket.key(), input_mint
    let bucket_key = bucket.key();
    let input_mint_key = ctx.accounts.input_mint.key();
    let vault_seeds = &[
        VAULT_SEED,
        bucket_key.as_ref(),
        input_mint_key.as_ref(),
        &[ctx.bumps.vault_input_token_account],
    ];
    let signer_seeds: &[&[&[u8]]] = &[&vault_seeds[..]];

    // Build account_infos slice: must contain the same accounts (in any order) as required by invoke
    let account_infos = &[
        ctx.accounts.amm.to_account_info(),
        ctx.accounts.amm_authority.to_account_info(),
        ctx.accounts.pool_coin_token_account.to_account_info(),
        ctx.accounts.pool_pc_token_account.to_account_info(),
        ctx.accounts.user_source_token_account.to_account_info(),
        ctx.accounts.user_destination_token_account.to_account_info(),
        ctx.accounts.user_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        ctx.accounts.raydium_amm_program.to_account_info(),
    ];

    // Execute CPI (signed by vault PDA)
    invoke_signed(&ix, account_infos, signer_seeds).map_err(|e| {
        msg!("Raydium CPI failed: {:?}", e);
        error!(ErrorCode::RaydiumCpiFailed)
    })?;

    // Persist TradeRecord account
    let trade_record = &mut ctx.accounts.trade_record;
    trade_record.pool_id = bucket.key();
    trade_record.trade_id = trade_record.trade_id.saturating_add(1);
    trade_record.timestamp = clock.unix_timestamp;
    trade_record.trade_type = crate::state::TradeType::Rebalance; // Or BuyToken/SellToken
    trade_record.from_token = ctx.accounts.input_mint.key();
    trade_record.to_token = ctx.accounts.output_mint.key();
    trade_record.amount_in = in_amount;
    trade_record.amount_out = quoted_out_amount; // best-effort: client-supplied min; you may fetch actual after
    trade_record.success = true;

    msg!(
        "✅ Raydium CP-swap executed successfully for bucket: {} | Input: {} {} | MinOut: {} {}",
        bucket.name,
        in_amount,
        ctx.accounts.input_mint.key(),
        quoted_out_amount,
        ctx.accounts.output_mint.key()
    );

    Ok(())
}
