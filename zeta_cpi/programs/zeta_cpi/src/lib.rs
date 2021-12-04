use anchor_lang::prelude::*;

pub mod context;
pub mod zeta_client;
pub mod zeta_context;
pub mod constants;
pub mod types;
use crate::context::*;
use crate::types::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod zeta_cpi {
    use super::*;

    pub fn create_margin_account<'info>(ctx: Context<CreateMarginAccountCaller<'info>>) -> ProgramResult {
        zeta_client::create_margin_account(ctx.accounts.zeta_program.clone(), ctx.accounts.create_margin_cpi_accounts.clone())
    }

    pub fn initialize_margin_account(ctx: Context<InitializeMarginAccountCaller>) -> ProgramResult {
        zeta_client::initialize_margin_account(ctx.accounts.zeta_program.clone(), ctx.accounts.initialize_margin_cpi_accounts.clone())
    }

    pub fn deposit(ctx: Context<DepositCaller>, amount: u64) -> ProgramResult {
        zeta_client::deposit(ctx.accounts.zeta_program.clone(), ctx.accounts.deposit_cpi_accounts.clone(), amount)
    }

    pub fn withdraw(ctx: Context<WithdrawCaller>, amount: u64) -> ProgramResult {
        zeta_client::withdraw(ctx.accounts.zeta_program.clone(), ctx.accounts.withdraw_cpi_accounts.clone(), amount)
    }

    pub fn initialize_open_orders(ctx: Context<InitializeOpenOrdersCaller>) -> ProgramResult {
        zeta_client::initialize_open_orders(ctx.accounts.zeta_program.clone(), ctx.accounts.initialize_open_orders_cpi_accounts.clone())
    }

    pub fn place_order(
        ctx: Context<PlaceOrderCaller>,         
        price: u64,
        size: u32,
        side: Side
    ) -> ProgramResult {
        zeta_client::place_order(ctx.accounts.zeta_program.clone(), ctx.accounts.place_order_cpi_accounts.clone(), price, size, side)
    }
}