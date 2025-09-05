use anchor_lang::prelude::*;
use crate::error::ErrorCode;

#[inline(never)]
pub fn validate_inputs(
    name: &String,
    token_mints: &Vec<Pubkey>,
    contribution_window_days: u32,
    trading_window_days: u32,
    creator_fee_percent: u16,
) -> Result<()> {
    require!(name.len() <= 64, ErrorCode::NameTooLong);
    require!(token_mints.len() >= 2, ErrorCode::InsufficientTokens);
    require!(token_mints.len() <= 3, ErrorCode::TooManyTokens);
    require!(
        contribution_window_days > 0 && contribution_window_days <= 30,
        ErrorCode::InvalidContributionWindow
    );
    require!(
        trading_window_days > 0 && trading_window_days <= 180,
        ErrorCode::InvalidTradingWindow
    );
    require!(creator_fee_percent <= 2000, ErrorCode::FeeTooHigh);

    let mut unique_tokens = token_mints.clone();
    unique_tokens.sort();
    unique_tokens.dedup();
    require!(
        unique_tokens.len() == token_mints.len(),
        ErrorCode::DuplicateTokens
    );
    Ok(())
}
