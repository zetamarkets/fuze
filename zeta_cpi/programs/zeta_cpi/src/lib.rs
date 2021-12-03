use anchor_lang::prelude::*;
use zeta_client;
use zeta_client::types::*;

pub mod context;
use crate::context::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod zeta_cpi {
    use super::*;

    pub fn create_margin_account<'info>(ctx: Context<CreateMarginAccountCaller<'info>>) -> ProgramResult {
        zeta_client::create_margin_account(ctx.accounts.zeta_program.clone(), ctx.accounts.into())
    }

    // pub fn initialize_margin_account(ctx: Context<InitializeMarginAccountCaller>) -> ProgramResult {
    //     zeta_client::initialize_margin_account(ctx)
    // }

    // pub fn deposit(ctx: Context<DepositCaller>, amount: u64) -> ProgramResult {
    //     zeta_client::deposit(ctx, amount)
    // }

    // pub fn withdraw(ctx: Context<WithdrawCaller>, amount: u64) -> ProgramResult {
    //     zeta_client::withdraw(ctx, amount)
    // }

    // pub fn initialize_open_orders(
    //     ctx: Context<InitializeOpenOrdersCaller>,
    // ) -> ProgramResult {
    //     zeta_client::initialize_open_orders(ctx)
    // }

    // pub fn place_order(
    //         ctx: Context<PlaceOrderCaller>,         
    //         price: u64,
    //         size: u32,
    //         side: Side
    //     ) -> ProgramResult {
    //     zeta_client::place_order(ctx, price, size, side)
    // }
}