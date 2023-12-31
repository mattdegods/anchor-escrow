use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ close_account, transfer, CloseAccount, Mint, Token, TokenAccount, Transfer },
};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker:  Signer<'info>,
    pub mint_a: Account<'info, Mint>,

    #[account(
        mut, 
        // constraints: guarantees the tokens coming in are the one we want
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: Account<'info, TokenAccount>,

    #[account(
        // set mutable with close constraint
        mut,
        close = maker,
        // check that there is a valid mint_a
        has_one = mint_a,
        // requires seed constraints otherwise anyone can hack your escrow
        seeds = [b"escrow".as_ref(), maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        // saved the bump
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        token::mint = mint_a,
        // derive from escrow
        token::authority = escrow,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    // since we're using spl tokens not lamports:
    pub token_program: Program<'info, Token>,
    // need this for ata
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl <'info>Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        let transfer_accounts = Transfer {
            // from the vault
            from: self.vault.to_account_info(),
            // back to maker's account (since taker hasn't deposited yet)
            to: self.maker_ata_a.to_account_info(),
            // authority is vault?
            authority: self.escrow.to_account_info()
        };

        // messy rust/anchor thing - just copy when needed
        let binding = self.escrow.seed.to_le_bytes().clone();

        let seeds = &[
            b"escrow", 
            self.maker.to_account_info().key.as_ref(),
            binding.as_ref(),
            &[self.escrow.bump]
            ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds
        );

        transfer(cpi_ctx, self.vault.amount)

    }

    pub fn close_vault(&mut self) -> Result<()> {

        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        // messy rust/anchor thing - just copy when needed
        let binding = self.escrow.seed.to_le_bytes().clone();

        let seeds = &[
            b"escrow", 
            self.maker.to_account_info().key.as_ref(),
            binding.as_ref(),
            &[self.escrow.bump]
            ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts, signer_seeds
        );

        close_account(cpi_ctx)
    }

}

