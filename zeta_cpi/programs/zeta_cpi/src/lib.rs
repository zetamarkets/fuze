
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

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
    fn deposit(
        ctx: Context<T>,
        amount: u64,
    ) -> ProgramResult;
    fn withdraw(
        ctx: Context<T>,
        amount: u64,
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

    pub fn deposit(ctx: Context<DepositCaller>, amount: u64) -> ProgramResult {
        let cpi_program = ctx.accounts.zeta_program.clone();
        let cpi_accounts = Deposit {
            state: ctx.accounts.state.clone(),
            zeta_group: ctx.accounts.zeta_group.clone(),
            margin_account: ctx.accounts.margin_account.clone(),
            vault: ctx.accounts.vault.clone(),
            user_token_account: ctx.accounts.user_token_account.clone(),
            authority: ctx.accounts.authority.clone(),
            token_program: ctx.accounts.token_program.clone(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        zeta_interface::deposit(cpi_ctx, amount)
    }

    pub fn withdraw(ctx: Context<WithdrawCaller>, amount: u64) -> ProgramResult {
        let cpi_program = ctx.accounts.zeta_program.clone();
        let cpi_accounts = Withdraw {
            zeta_group: ctx.accounts.zeta_group.clone(),
            state: ctx.accounts.state.clone(),
            vault: ctx.accounts.vault.clone(),
            margin_account: ctx.accounts.margin_account.clone(),
            user_token_account: ctx.accounts.user_token_account.clone(),
            token_program: ctx.accounts.token_program.clone(),
            authority: ctx.accounts.authority.clone(),
            greeks: ctx.accounts.greeks.clone(),
            oracle: ctx.accounts.oracle.clone(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        zeta_interface::withdraw(cpi_ctx, amount)
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

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Deposit<'info> {
    pub state: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Withdraw<'info> {
    pub zeta_group: AccountInfo<'info>,
    pub state: AccountInfo<'info>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub greeks: AccountInfo<'info>,
    pub oracle: AccountInfo<'info>,
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

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct DepositCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub state: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct WithdrawCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    pub state: AccountInfo<'info>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub greeks: AccountInfo<'info>,
    pub oracle: AccountInfo<'info>,
}