use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Raydium CPI failed")]
    RaydiumCpiFailed,
    #[msg("Bucket name is too long (max 64 characters)")]
    NameTooLong,
    #[msg("Must have at least 2 tokens")]
    InsufficientTokens,
    #[msg("Too many tokens (max 10)")]
    TooManyTokens,
    #[msg("Invalid contribution window (1 minute to 30 days)")]
    InvalidContributionWindow,
    #[msg("Invalid trading window (1 minute to 180 days)")]
    InvalidTradingWindow,
    #[msg("Creator fee too high (max 20%)")]
    FeeTooHigh,
    #[msg("Duplicate tokens not allowed")]
    DuplicateTokens,
    #[msg("Bucket is not in raising state")]
    BucketNotRaising,
    #[msg("Contribution deadline has passed")]
    ContributionDeadlinePassed,
    #[msg("Token not allowed in this bucket")]
    TokenNotAllowed,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Unauthorized creator")]
    UnauthorizedCreator,
    #[msg("Contribution window still active")]
    ContributionStillActive,
    #[msg("No contributions made")]
    NoContributions,
    #[msg("Bucket is not in trading state")]
    BucketNotTrading,
    #[msg("Trading window still active")]
    TradingStillActive,
    #[msg("Bucket is not closed")]
    BucketNotClosed,
    /// ## @audit: This error is not used in the codebase especially in the contribute function
    #[msg("Unauthorized contributor")]
    UnauthorizedContributor,
    #[msg("No rewards available")]
    NoRewardsAvailable,
    #[msg("Trading has not started yet")]
    TradingNotStarted,
    #[msg("Invalid token mint for this bucket")]
    InvalidTokenMint,
    #[msg("Trading deadline has passed")]
    TradingDeadlinePassed,
    #[msg("Invalid bucket status")]
    InvalidBucketStatus,
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,
    #[msg("Invalid swap amount")]
    InvalidSwapAmount,
    #[msg("Overflow occurred during arithmetic operation")]
    Overflow,
    #[msg("Profile already exists for this creator")]
    ProfileAlreadyExists,
    #[msg("Profile does not exist for this creator")]
    ProfileDoesNotExist,
    #[msg("Program not initialized")]
    ProgramNotInitialized,
    #[msg("Program already initialized")]
    ProgramInitialized, 
    #[msg("Invalid mint address")]
    InvalidMint,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Only the deployer can initialize the program")]
    UnauthorizedDeployer,
}
