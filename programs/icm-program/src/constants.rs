use anchor_lang::prelude::*;
use std::str::FromStr;

// USDC Mint Address (mainnet, update if needed for devnet/testnet)
pub const USDC_MINT: &str = "2RgRJx3z426TMCL84ZMXTRVCS5ee7iGVE4ogqcUAd3tg";

pub const VAULT_SEED: &[u8] = b"vault";

pub fn jupiter_program_id() -> Pubkey {
    // Jupiter v6 program ID
    Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap()
}
