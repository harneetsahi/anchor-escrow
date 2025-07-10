use anchor_lang::prelude::*;

#[account] // tells anchor to automatically implement serialization/deserialization for this struct so it can be stored on-chain.
#[derive(InitSpace)]
pub struct Escrow {
  pub seed: u64,
  pub maker: Pubkey,
  pub mint_a: Pubkey,
  pub mint_b: Pubkey,
  pub receive: u64,
  pub bump: u8
} // these are internal data fields that reside inside the escrow account's space on chain. These are the fields our program will read and write.

