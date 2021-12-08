use anchor_lang::prelude::*;
use rust_decimal::prelude::*;

pub mod context;
pub mod pyth_client;
pub mod zeta_account;
pub mod zeta_client;
pub mod zeta_constants;
pub mod zeta_context;
pub mod zeta_utils;
use crate::context::*;
use crate::zeta_account::*;
use crate::zeta_constants::*;
use crate::zeta_utils::*;

declare_id!("7Aqh4kCtp3rdtHSk1mFXswyw37z9Ldad7vmgrbVD2h9J");

#[program]
pub mod zeta_cpi {
    use super::*;

    pub fn initialize_margin_account(ctx: Context<InitializeMarginAccountCaller>) -> ProgramResult {
        zeta_client::initialize_margin_account(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.initialize_margin_cpi_accounts.clone(),
        )
    }

    pub fn deposit(ctx: Context<DepositCaller>, amount: u64) -> ProgramResult {
        zeta_client::deposit(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.deposit_cpi_accounts.clone(),
            amount,
        )
    }

    pub fn withdraw(ctx: Context<WithdrawCaller>, amount: u64) -> ProgramResult {
        zeta_client::withdraw(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.withdraw_cpi_accounts.clone(),
            amount,
        )
    }

    pub fn initialize_open_orders(ctx: Context<InitializeOpenOrdersCaller>) -> ProgramResult {
        zeta_client::initialize_open_orders(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.initialize_open_orders_cpi_accounts.clone(),
        )
    }

    pub fn place_order(
        ctx: Context<PlaceOrderCaller>,
        price: u64,
        size: u32,
        side: Side,
    ) -> ProgramResult {
        zeta_client::place_order(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.place_order_cpi_accounts.clone(),
            price,
            size,
            side,
        )
    }

    pub fn cancel_order(
        ctx: Context<CancelOrderCaller>,
        side: Side,
        order_id: u128,
    ) -> ProgramResult {
        zeta_client::cancel_order(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.cancel_order_cpi_accounts.clone(),
            side,
            order_id,
        )
    }

    pub fn read_program_data(ctx: Context<ReadProgramData>) -> ProgramResult {
        let zeta_group =
            deserialize_account_info_zerocopy::<ZetaGroup>(&ctx.accounts.zeta_group).unwrap();

        // Get the data for the front expiration.
        let front_expiry_index = zeta_group.front_expiry_index as usize;
        let expiry_series = zeta_group.expiry_series[front_expiry_index];
        {
            // The unix timestamp that the products are tradeable.
            msg!("Active timestamp {}", expiry_series.active_ts);

            // The unix timestamp that the products expire.
            msg!("Expiry timestamp {}", expiry_series.expiry_ts);

            let status = zeta_group.expiry_series[front_expiry_index].status()?;

            // If the market is tradeable.
            msg!("Is market live?: {:?}", status == ExpirySeriesStatus::Live);

            // If the market has expired.
            msg!(
                "Is market expired?: {:?}",
                status == ExpirySeriesStatus::Expired
            );
        }

        // Show the data for all products in a given expiry series. Use the front expiry.
        let products_slice = zeta_group.get_products_slice(front_expiry_index);
        for i in 0..products_slice.len() {
            let product = &products_slice[i];

            // The market index of the given product.
            // This allows for direct indexing into zeta_group.
            // i.e `product == &zeta_group.products[product_index]`
            let market_index = get_products_slice_market_index(front_expiry_index, i);

            // Strike has 6 decimals of precision.
            let strike = product.strike.get_strike()?;

            // The serum market this product trades on.
            let market = product.market;

            // Call / Put / Future
            let kind = product.kind;

            msg!(&format!(
                "Market index = {}, Strike = {}, Market = {}, Kind = {:?}",
                market_index, strike, market, kind
            ));
        }

        // This returns the oracle price as a fixed point integer with 6 decimals of precision
        let oracle_price = get_native_oracle_price(&ctx.accounts.oracle);
        msg!("Oracle price {:?}", oracle_price);

        // Get the mark price and greek data for the first product in the expiry series.
        // This happens to be the lowest strike call.
        let market_index = get_products_slice_market_index(front_expiry_index, 0);
        let greeks_index = get_greeks_index(front_expiry_index, market_index);

        let greeks = deserialize_account_info_zerocopy::<Greeks>(&ctx.accounts.greeks).unwrap();

        msg!(&format!(
            "Market index = {}, Mark price = {}, Delta = {}, Vega = {:?}, IV = {:?}",
            market_index,
            greeks.mark_prices[market_index],
            greeks.product_greeks[greeks_index].delta,
            Decimal::from(greeks.product_greeks[greeks_index].vega),
            Decimal::from(greeks.product_greeks[greeks_index].volatility)
        ));

        let margin_account =
            deserialize_account_info_zerocopy::<MarginAccount>(&ctx.accounts.margin_account)
                .unwrap();

        msg!("Margin account balance: {:?}", margin_account.balance);

        // Position details for a given market index.
        let position = margin_account.positions[market_index].position;
        let cost_of_trades = margin_account.positions[market_index].cost_of_trades;

        msg!(
            "Margin account position for market index {}: Position={}, Cost of trades={}",
            market_index,
            position,
            cost_of_trades
        );

        let initial_margin_requirement =
            margin_account.get_initial_margin(&greeks, &zeta_group, oracle_price);
        let maintenance_margin_requirement =
            margin_account.get_maintenance_margin(&greeks, &zeta_group, oracle_price);
        let total_margin_requirement = initial_margin_requirement
            .checked_add(maintenance_margin_requirement)
            .unwrap();

        msg!(
            "Margin account: Initial: {}, Maintenance: {}, Total: {}",
            initial_margin_requirement,
            maintenance_margin_requirement,
            total_margin_requirement
        );

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
    ProductDirty,
}
