use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use jupiter_interface::{
    instructions::{RouteIxArgs, RouteKeys},
    typedefs::RoutePlanStep,
};
use crate::{constants::*, error::ErrorCode, state::*};

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
    pub input_mint_program: Interface<'info, TokenInterface>,
    pub output_mint: InterfaceAccount<'info, Mint>,
    pub output_mint_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
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

    /// CHECK: Jupiter program
    pub jupiter_program: UncheckedAccount<'info>,

    /// Token program for Token-2022 compatibility
    pub token_2022_program: Interface<'info, TokenInterface>,

    /// Platform fee account (optional - can be same as vault_output_token_account)
    #[account(mut)]
    pub platform_fee_account: InterfaceAccount<'info, TokenAccount>,
}

pub fn swap_tokens_handler(
    ctx: Context<SwapTokens>,
    route_plan: Vec<u8>,
    in_amount: u64,
    quoted_out_amount: u64,
    slippage_bps: u16,
    platform_fee_bps: u16,
) -> Result<()> {
    let bucket = &ctx.accounts.bucket;

    // Convert route_plan from Vec<u8> to Vec<RoutePlanStep>
    // Note: This assumes the route_plan bytes can be deserialized
    let route_plan_steps: Vec<RoutePlanStep> = match route_plan.len() {
        0 => vec![], // Empty route plan
        _ => {
            // For now, we'll create a simple route plan step
            // In a real implementation, you'd deserialize the bytes properly
            vec![]
        }
    };

    // Convert platform_fee_bps from u16 to u8 (with bounds checking)
    let platform_fee_bps_u8 = if platform_fee_bps > 255 {
        return Err(ErrorCode::InvalidSwapAmount.into());
    } else {
        platform_fee_bps as u8
    };

    // === VALIDATION CHECKS ===

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

    // === JUPITER CPI SWAP ===

    // Prepare signer seeds for the vault authority
    let bucket_key = ctx.accounts.bucket.key();
    let input_mint_key = ctx.accounts.input_mint.key();

    let vault_seeds = &[
        VAULT_SEED,
        bucket_key.as_ref(),
        input_mint_key.as_ref(),
        &[ctx.bumps.vault_input_token_account],
    ];
    let signer_seeds: &[&[&[u8]]] = &[&vault_seeds[..]];

    // Create the route keys for Jupiter
    let route_keys = RouteKeys {
        token_program: ctx.accounts.input_mint_program.key(),
        user_transfer_authority: ctx.accounts.vault_input_token_account.key(), // Vault acts as authority
        user_source_token_account: ctx.accounts.vault_input_token_account.key(),
        user_destination_token_account: ctx.accounts.vault_output_token_account.key(),
        program: ctx.accounts.jupiter_program.key(),
        platform_fee_account: ctx.accounts.platform_fee_account.key(),
        destination_token_account: ctx.accounts.vault_output_token_account.key(),
        destination_mint: ctx.accounts.output_mint.key(),
        event_authority: Pubkey::default(), // Use default if not needed
    };

    // Create the route arguments
    let route_args = RouteIxArgs {
        route_plan: route_plan_steps,
        in_amount,
        quoted_out_amount,
        slippage_bps,
        platform_fee_bps: platform_fee_bps_u8,
    };

    // Create the Jupiter route instruction
    let ix = jupiter_interface::instructions::route_ix(route_keys, route_args)?;

    // Execute the swap via CPI with signer seeds
    invoke_signed(
        &ix,
        &[
            ctx.accounts.jupiter_program.to_account_info(),
            ctx.accounts.vault_input_token_account.to_account_info(),
            ctx.accounts.vault_output_token_account.to_account_info(),
            ctx.accounts.input_mint_program.to_account_info(),
            ctx.accounts.platform_fee_account.to_account_info(),
        ],
        signer_seeds,
    )?;

    // Persist TradeRecord account
    let trade_record = &mut ctx.accounts.trade_record;
    trade_record.pool_id = bucket.key();
    trade_record.trade_id += 1;
    trade_record.timestamp = clock.unix_timestamp;
    trade_record.trade_type = crate::state::TradeType::Rebalance; // Or BuyToken/SellToken
    trade_record.from_token = ctx.accounts.input_mint.key();
    trade_record.to_token = ctx.accounts.output_mint.key();
    trade_record.amount_in = in_amount;
    trade_record.amount_out = quoted_out_amount;
    trade_record.success = true;

    msg!(
        "✅ Jupiter swap executed successfully for bucket: {} | Input: {} {} | Expected output: {} {}",
        bucket.name,
        in_amount,
        ctx.accounts.input_mint.key(),
        quoted_out_amount,
        ctx.accounts.output_mint.key()
    );

    Ok(())
}
