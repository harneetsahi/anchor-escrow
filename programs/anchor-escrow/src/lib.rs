#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HbP6Y6ZKEaVdCVW9zAVMEch1mYo1rXbh4mZ7ZyEAugVf");

#[program]
pub mod anchor_escrow {
    use super::*;

    pub fn init_escrow(ctx: Context<MakeOffer>, seed: u64, receive: u64 ) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)
    }

    pub fn deposit(ctx: Context<MakeOffer>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn take_offer(ctx: Context<TakeOffer>, amount: u64) -> Result<()> {
        ctx.accounts.take_offer(amount)
    }
}

