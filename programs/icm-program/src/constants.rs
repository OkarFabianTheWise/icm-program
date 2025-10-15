use anchor_lang::prelude::*;
use std::str::FromStr;

/// USDC Mint Address (mainnet, update if needed for devnet/testnet)
pub const USDC_MINT: &str = "2RgRJx3z426TMCL84ZMXTRVCS5ee7iGVE4ogqcUAd3tg";
pub fn usdc_id() -> Pubkey{
    Pubkey::from_str(USDC_MINT).unwrap()
}

/// Vault seed
pub const VAULT_SEED: &[u8] = b"vault";

/// Jupiter Agg. v6 program ID
pub fn jupiter_program_id() -> Pubkey {
    Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap()
}
