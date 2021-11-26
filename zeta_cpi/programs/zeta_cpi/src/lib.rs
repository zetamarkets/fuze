
use anchor_lang::prelude::*;
pub mod constants;
// pub mod context;
use crate::constants::*;
// use context::{InitializeMarginAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[interface]
pub trait Zeta<'info, T: Accounts<'info>> {
    fn initialize_margin_account(
        ctx: Context<T>,
        nonce: u8,
    ) -> ProgramResult;
}

#[program]
pub mod zeta_cpi {
    use super::*;

    pub fn initialize_margin_account(ctx: Context<InitializeMarginAccount>) -> ProgramResult {
        let cpi_program = ctx.accounts.zeta_program.clone();
        let cpi_accounts = InitializeMarginAccount {
            zeta_program: cpi_program.clone(),
            zeta_group: ctx.accounts.zeta_group.to_account_info(),
            margin_account: ctx.accounts.margin_account.to_account_info(),
            authority: ctx.accounts.authority.clone(),
            system_program: ctx.accounts.system_program.clone(),
        };
        let (_pda, nonce)  = Pubkey::find_program_address(&[MARGIN_SEED.as_bytes()], &cpi_program.key.clone());
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        // Zeta::<InitializeMarginAccount>::initialize_margin_account(cpi_ctx, nonce);
        zeta::initialize_margin_account(cpi_ctx, nonce)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Empty {}

#[derive(Accounts)]
pub struct InitializeMarginAccount<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    pub margin_account: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}