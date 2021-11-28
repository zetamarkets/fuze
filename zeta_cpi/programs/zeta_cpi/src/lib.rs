
use anchor_lang::prelude::*;
pub mod constants;
use crate::constants::*;
use cpi_interface::global_interface;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[global_interface]
pub trait ZetaInterface<'info, T: Accounts<'info>> {
    fn create_margin_account(
        ctx: Context<T>,
        nonce: u8,
    ) -> ProgramResult;
    fn initialize_margin_account(
        ctx: Context<T>,
        nonce: u8,
    ) -> ProgramResult;
}

#[program]
pub mod zeta_cpi {
    use super::*;

    pub fn create_margin_account(ctx: Context<CreateMarginAccountCaller>) -> ProgramResult {
        let cpi_program = ctx.accounts.zeta_program.clone();
        let cpi_accounts = CreateMarginAccount {
            margin_account: ctx.accounts.margin_account.clone(),
            authority: ctx.accounts.authority.clone(),
            system_program: ctx.accounts.system_program.clone(),
            zeta_program: cpi_program.clone(),
            zeta_group: ctx.accounts.zeta_group.clone(),
        };
        let (_pda, nonce)  = Pubkey::find_program_address(&[MARGIN_SEED.as_bytes(), ctx.accounts.zeta_group.key.as_ref(), ctx.accounts.authority.key.as_ref()], &cpi_program.key.clone());
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        zeta_interface::create_margin_account(cpi_ctx, nonce)
    }

    pub fn initialize_margin_account(ctx: Context<InitializeMarginAccountCaller>) -> ProgramResult {
        let cpi_program = ctx.accounts.zeta_program.clone();
        let cpi_accounts = InitializeMarginAccount {
            zeta_group: ctx.accounts.zeta_group.clone(),
            margin_account: ctx.accounts.margin_account.clone(),
            authority: ctx.accounts.authority.clone(),
            system_program: ctx.accounts.system_program.clone(),
        };
        let (_pda, nonce)  = Pubkey::find_program_address(&[MARGIN_SEED.as_bytes(), ctx.accounts.zeta_group.key.as_ref(), ctx.accounts.authority.key.as_ref()], &cpi_program.key.clone());
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        zeta_interface::initialize_margin_account(cpi_ctx, nonce)
    }
}

#[derive(Accounts)]
pub struct CreateMarginAccount<'info> {
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(address = ID)]
    pub zeta_program: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]

pub struct InitializeMarginAccount<'info> {
    pub zeta_group: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [MARGIN_SEED.as_bytes().as_ref(), zeta_group.key.as_ref(), authority.key.as_ref()],
        bump = nonce,
    )]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Dummy caller fns

#[derive(Accounts)]
pub struct CreateMarginAccountCaller<'info> {
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub zeta_program: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
}

// Note to self: don't do seeds validation in the outer call because that will use the calling programid
#[derive(Accounts)]
pub struct InitializeMarginAccountCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}