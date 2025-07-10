use anchor_lang::prelude::*;
use crate::state::Escrow;
use anchor_spl::{
 associated_token::AssociatedToken,
 token::{close_account, transfer_checked, CloseAccount, TransferChecked}, token_interface:: {Mint, TokenAccount, TokenInterface}
};



#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct TakeOffer<'info> {

  #[account(mut)]
  pub taker: Signer<'info>,

  #[account(mut)]
  pub maker: SystemAccount<'info>,

  #[account(
    mint::token_program = token_program
  )]
  pub mint_a: InterfaceAccount<'info, Mint>,

  #[account(
    mint::token_program = token_program
  )]
  pub mint_b: InterfaceAccount<'info, Mint>,

  #[account(
    init_if_needed,
    payer = taker,
    associated_token::mint = mint_a,
    associated_token::authority = taker,
    associated_token::token_program = token_program
  )]
  pub taker_ata_a: InterfaceAccount<'info, TokenAccount>, // this is where we put tokens we get from the vault

  #[account(
    mut,
    associated_token::mint = mint_b,
    associated_token::authority = taker,
    associated_token::token_program = token_program
  )]
  pub taker_ata_b: InterfaceAccount<'info, TokenAccount>, // this is where we take tokens from to send them to the maker

   #[account(
    init_if_needed,
    payer = taker,
    associated_token::mint = mint_b,
    associated_token::authority = maker,
    associated_token::token_program = token_program
  )]
  pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

  #[account(
    mut,
    close = maker,
    has_one = maker,
    has_one = mint_a,
    has_one = mint_b,
    seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
    bump = escrow.bump
  )]
  pub escrow: Account<'info, Escrow>,


  #[account(
    mut,
    associated_token::mint = mint_a, // token a is what was stored in the vault
    associated_token::authority = escrow,
    associated_token::token_program = token_program
  )]
  pub vault: InterfaceAccount<'info, TokenAccount>, 


  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>

}


impl<'info> TakeOffer<'info> {

  pub fn take_offer(&mut self, amount: u64) -> Result<()> {

      let escrow_seeds = &[
        b"escrow".as_ref(),
        self.maker.to_account_info().key.as_ref(),
        &self.escrow.seed.to_le_bytes()[..],
        &[self.escrow.bump]
      ];

      let signer_seeds = &[&escrow_seeds[..]];


      // transfer from vault to taker
      let transfer_accounts_to_taker = TransferChecked{
        from: self.vault.to_account_info(),
        to: self.taker_ata_a.to_account_info(),
        mint: self.mint_a.to_account_info(),
        authority: self.escrow.to_account_info()
      };

      let cpi_ctx_to_taker = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_accounts_to_taker, signer_seeds);

      transfer_checked(cpi_ctx_to_taker, self.vault.amount, self.mint_a.decimals)?;


      // close vault
      let close_accounts = CloseAccount{
        account: self.vault.to_account_info(),
        destination: self.maker.to_account_info(),
        authority: self.escrow.to_account_info(),
      };

      let close_cpi = CpiContext::new_with_signer(self.token_program.to_account_info(), close_accounts, signer_seeds);

      close_account(close_cpi)?;


      // transfer from taker to maker
      let transfer_accounts_to_maker = TransferChecked {
        from: self.taker_ata_b.to_account_info(),
        to: self.maker_ata_b.to_account_info(),
        mint: self.mint_b.to_account_info(),
        authority: self.taker.to_account_info()
      };

      let cpi_ctx_to_maker = CpiContext::new(self.token_program.to_account_info(), transfer_accounts_to_maker);

      transfer_checked(cpi_ctx_to_maker, amount, self.mint_b.decimals)?;

      Ok(())
      
  }

}