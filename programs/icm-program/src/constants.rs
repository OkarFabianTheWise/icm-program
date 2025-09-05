use anchor_lang::prelude::*;
use std::str::FromStr;

pub const VAULT_SEED: &[u8] = b"vault";

pub fn jupiter_program_id() -> Pubkey {
    // Jupiter v6 program ID
    Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap()
}
