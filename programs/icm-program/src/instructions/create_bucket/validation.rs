use crate::error::ErrorCode;
use anchor_lang::prelude::*;

#[inline(never)]
pub fn validate_inputs(
    name: &String,
    token_mints: &Vec<Pubkey>,
    contribution_window_minutes: u32,
    trading_window_minutes: u32,
    creator_fee_percent: u16,
) -> Result<()> {
    // use 50 as in the max_len and error code for easy debugging
    require!(name.len() <= 50, ErrorCode::NameTooLong);
    require!(token_mints.len() >= 2, ErrorCode::InsufficientTokens);
    require!(token_mints.len() <= 3, ErrorCode::TooManyTokens);
    require!(
        contribution_window_minutes >= 1 && contribution_window_minutes <= 43200, // 1 minute to 30 days (43200 minutes)
        ErrorCode::InvalidContributionWindow
    );
    require!(
        trading_window_minutes >= 1 && trading_window_minutes <= 259200, // 1 minute to 180 days (259200 minutes)
        ErrorCode::InvalidTradingWindow
    );
    require!(creator_fee_percent <= 2000, ErrorCode::FeeTooHigh); // 20 percent

    let mut unique_tokens = token_mints.clone();
    unique_tokens.sort();
    unique_tokens.dedup();
    require!(
        unique_tokens.len() == token_mints.len(),
        ErrorCode::DuplicateTokens
    );
    Ok(())
}
