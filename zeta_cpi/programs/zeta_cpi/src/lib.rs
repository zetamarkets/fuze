use anchor_lang::prelude::*;
use rust_decimal::prelude::*;

pub mod context;
pub mod zeta_client;
pub mod zeta_context;
pub mod constants;
pub mod utils;
pub mod zeta_account;
pub mod pc;
use crate::context::*;
use crate::utils::*;
use crate::zeta_account::*;
use crate::constants::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod zeta_cpi {
    use super::*;

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

    pub fn read_program_data(ctx: Context<ReadProgramData>) -> ProgramResult {
        let state = deserialize_account_info::<State>(&ctx.accounts.state).unwrap();
        msg!("state.num_underlyings: {:?}", state.num_underlyings);

        let zeta_group = deserialize_account_info_zerocopy::<ZetaGroup>(&ctx.accounts.zeta_group).unwrap();
        let front_expiry_index = zeta_group.front_expiry_index as usize;
        msg!("Market strike: {:?}", zeta_group.products[front_expiry_index].strike.get_strike().unwrap());
        msg!("Market expiry: {:?}", zeta_group.expiry_series[front_expiry_index].expiry_ts);
        msg!("Is market expired?: {:?}", zeta_group.expiry_series[front_expiry_index].status().unwrap() == ExpirySeriesStatus::Expired);
        
        let oracle_price = get_native_oracle_price(&ctx.accounts.oracle);
        msg!("Oracle price {:?}", oracle_price);
        
        let greeks = deserialize_account_info_zerocopy::<Greeks>(&ctx.accounts.greeks).unwrap();
        msg!("Mark price: {:?}", greeks.mark_prices[front_expiry_index]);
        msg!("Delta: {:?}", greeks.product_greeks[front_expiry_index].delta);
        msg!("Vega: {:?}", Decimal::from(greeks.product_greeks[front_expiry_index].vega));
        msg!("IV: {:?}", Decimal::from(greeks.product_greeks[front_expiry_index].volatility));
        
        let margin_account = deserialize_account_info_zerocopy::<MarginAccount>(&ctx.accounts.margin_account).unwrap();
        msg!("Margin balance: {:?}", margin_account.balance);
        msg!("Market net position: {:?}", margin_account.positions[front_expiry_index].position);
        msg!("Market initial margin: {:?}", margin_account.get_initial_margin(&greeks, &zeta_group, oracle_price));
        msg!("Market maintenance margin: {:?}", margin_account.get_maintenance_margin(&greeks, &zeta_group, oracle_price));

        Ok(())
    }
}

#[error]
pub enum ErrorCode {
    #[msg("Account not mutable")]
    AccountNotMutable,
    #[msg("Unsupported kind")]
    UnsupportedKind,
    #[msg("Product strike uninitialized")]
    ProductStrikeUninitialized,
    #[msg("Invalid product market key")]
    InvalidProductMarketKey,
    #[msg("Market not live")]
    MarketNotLive,
    #[msg("Product dirty")]
    ProductDirty
}