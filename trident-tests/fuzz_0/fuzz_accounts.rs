use trident_fuzz::fuzzing::*;

/// FuzzAccounts contains all available accounts
///
/// You can create your own accounts by adding new fields to the struct.
///
/// Docs: https://ackee.xyz/trident/docs/latest/trident-api-macro/trident-types/fuzz-accounts/
#[derive(Default)]
pub struct FuzzAccounts {
    pub creator_profile: AccountsStorage,

    pub vault_input_token_account: AccountsStorage,

    pub system_program: AccountsStorage,

    pub raydium_amm_program: AccountsStorage,

    pub contributor_token_account: AccountsStorage,

    pub user_authority: AccountsStorage,

    pub pool_coin_token_account: AccountsStorage,

    pub pyth_oracle: AccountsStorage,

    pub reserve_collateral_mint: AccountsStorage,

    pub reserve: AccountsStorage,

    pub creator: AccountsStorage,

    pub bucket: AccountsStorage,

    pub trade_record: AccountsStorage,

    pub input_mint: AccountsStorage,

    pub usdc_mint: AccountsStorage,

    pub lending_market: AccountsStorage,

    pub fee_vault: AccountsStorage,

    pub program_state: AccountsStorage,

    pub owner: AccountsStorage,

    pub source_liquidity: AccountsStorage,

    pub switchboard_oracle: AccountsStorage,

    pub vault_token_account: AccountsStorage,

    pub pool_pc_token_account: AccountsStorage,

    pub user_source_token_account: AccountsStorage,

    pub contribution_record: AccountsStorage,

    pub amm_authority: AccountsStorage,

    pub rent: AccountsStorage,

    pub associated_token_program: AccountsStorage,

    pub lending_market_authority: AccountsStorage,

    pub destination_collateral: AccountsStorage,

    pub output_mint: AccountsStorage,

    pub owner_token_account: AccountsStorage,

    pub pool_contribution: AccountsStorage,

    pub solend_program: AccountsStorage,

    pub contributor: AccountsStorage,

    pub trading_pool: AccountsStorage,

    pub reserve_liquidity_supply: AccountsStorage,

    pub token_program: AccountsStorage,

    pub output_mint_program: AccountsStorage,

    pub input_mint_program: AccountsStorage,

    pub vault_output_token_account: AccountsStorage,

    pub user_destination_token_account: AccountsStorage,

    pub amm: AccountsStorage,
}
