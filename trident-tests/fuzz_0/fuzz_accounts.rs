use trident_fuzz::fuzzing::*;

/// FuzzAccounts contains all available accounts
///
/// You can create your own accounts by adding new fields to the struct.
///
/// Docs: https://ackee.xyz/trident/docs/latest/trident-api-macro/trident-types/fuzz-accounts/
#[derive(Default)]
pub struct FuzzAccounts {
    pub reserve: AccountsStorage,

    pub lending_market_authority: AccountsStorage,

    pub usdc_mint: AccountsStorage,

    pub pool_pc_token_account: AccountsStorage,

    pub bucket: AccountsStorage,

    pub lending_market: AccountsStorage,

    pub trading_pool: AccountsStorage,

    pub owner: AccountsStorage,

    pub contributor: AccountsStorage,

    pub fee_vault: AccountsStorage,

    pub system_program: AccountsStorage,

    pub source_liquidity: AccountsStorage,

    pub vault_token_account: AccountsStorage,

    pub contributor_token_account: AccountsStorage,

    pub trade_record: AccountsStorage,

    pub vault_input_token_account: AccountsStorage,

    pub pool_coin_token_account: AccountsStorage,

    pub rent: AccountsStorage,

    pub user_destination_token_account: AccountsStorage,

    pub owner_token_account: AccountsStorage,

    pub destination_collateral: AccountsStorage,

    pub reserve_liquidity_supply: AccountsStorage,

    pub reserve_collateral_mint: AccountsStorage,

    pub creator: AccountsStorage,

    pub input_mint: AccountsStorage,

    pub pyth_oracle: AccountsStorage,

    pub program_state: AccountsStorage,

    pub raydium_amm_program: AccountsStorage,

    pub creator_profile: AccountsStorage,

    pub token_program: AccountsStorage,

    pub contribution_record: AccountsStorage,

    pub associated_token_program: AccountsStorage,

    pub user_source_token_account: AccountsStorage,

    pub amm: AccountsStorage,

    pub solend_program: AccountsStorage,

    pub output_mint: AccountsStorage,

    pub switchboard_oracle: AccountsStorage,

    pub user_authority: AccountsStorage,

    pub input_mint_program: AccountsStorage,

    pub amm_authority: AccountsStorage,

    pub output_mint_program: AccountsStorage,

    pub vault_output_token_account: AccountsStorage,

    pub pool_contribution: AccountsStorage,
}
