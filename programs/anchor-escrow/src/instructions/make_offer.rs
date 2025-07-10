use anchor_lang::prelude::*;
use crate::state::Escrow;
use anchor_spl::{
 associated_token::AssociatedToken, token::{transfer_checked, TransferChecked}, token_interface:: {Mint, TokenAccount, TokenInterface}
};


#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct MakeOffer<'info> {

  #[account(mut)]
  pub maker: Signer<'info>,

  #[account(
    mint::token_program = token_program // this verifies that mint account is a valid SPL token mint and is owned by the token program we specified below 
  )]
  pub mint_a : InterfaceAccount<'info, Mint>,

  #[account(
    mint::token_program = token_program
  )]
  pub mint_b: InterfaceAccount<'info, Mint>,

  #[account(
    mut,
    associated_token::mint = mint_a,
    associated_token::authority = maker,
    associated_token::token_program = token_program
  )]
  pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

  #[account(
    init,
    payer = maker,
    space = 8 + Escrow::INIT_SPACE,
    seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
    bump,
  )] // external properties of the on-chain account itself. These are not stored inside the Escrow, but they tell anchor how to manage the lifecycle and properties of escrow account on chain
  pub escrow: Account<'info, Escrow>, 

  #[account(
    init,
    payer = maker,
    associated_token::mint = mint_a,
    associated_token::authority = escrow,
    associated_token::token_program = token_program,
  )]
  pub vault: InterfaceAccount<'info, TokenAccount>,

  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Interface<'info, TokenInterface>, // TokenInterface allows us to use both token and token22 program
  pub system_program: Program<'info, System>

}

impl<'info> MakeOffer<'info> {

  pub fn init_escrow(&mut self, seed: u64, receive: u64, bumps: &MakeOfferBumps) -> Result<()> {

     self.escrow.set_inner(
      Escrow {
        seed,
        maker: self.maker.key(),
        mint_a: self.mint_a.key(),
        mint_b: self.mint_b.key(),
        receive,
        bump: bumps.escrow
      }
     );

    Ok(())
  }

  pub fn deposit(&mut self, amount: u64) -> Result <()> {

    let cpi_program = self.token_program.to_account_info();

    let transfer_accounts = TransferChecked {
      from: self.maker_ata_a.to_account_info(),
      to: self.vault.to_account_info(),
      mint: self.mint_a.to_account_info(),
      authority: self.maker.to_account_info()
    };

    let cpi_ctx = CpiContext::new(cpi_program, transfer_accounts);

    transfer_checked(cpi_ctx, amount, self.mint_a.decimals)
  }
}