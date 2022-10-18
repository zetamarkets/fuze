use anchor_lang::prelude::*;
use rust_decimal::prelude::*;

pub mod context;
pub mod pyth_client;
pub mod zeta_account;
pub mod zeta_calculations;
pub mod zeta_client;
pub mod zeta_constants;
pub mod zeta_context;
pub mod zeta_utils;
use crate::context::*;
use crate::zeta_account::*;
use crate::zeta_calculations::*;
use crate::zeta_constants::*;
use crate::zeta_utils::*;

#[cfg(feature = "devnet")]
declare_id!("BG3oRikW8d16YjUEmX3ZxHm9SiJzrGtMhsSR8aCw1Cd7");

#[cfg(not(feature = "devnet"))]
declare_id!("ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD");

#[program]
pub mod zeta_cpi {
    use super::*;

    pub fn initialize_spread_account(ctx: Context<InitializeSpreadAccountCaller>) -> Result<()> {
        zeta_client::initialize_spread_account(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.initialize_spread_cpi_accounts.clone(),
            None,
        )
    }

    pub fn initialize_margin_account(ctx: Context<InitializeMarginAccountCaller>) -> Result<()> {
        zeta_client::initialize_margin_account(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.initialize_margin_cpi_accounts.clone(),
            None,
        )
    }

    pub fn deposit(ctx: Context<DepositCaller>, amount: u64) -> Result<()> {
        zeta_client::deposit(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.deposit_cpi_accounts.clone(),
            None,
            amount,
        )
    }

    pub fn withdraw(ctx: Context<WithdrawCaller>, amount: u64) -> Result<()> {
        zeta_client::withdraw(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.withdraw_cpi_accounts.clone(),
            None,
            amount,
        )
    }

    pub fn initialize_open_orders(ctx: Context<InitializeOpenOrdersCaller>) -> Result<()> {
        zeta_client::initialize_open_orders(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.initialize_open_orders_cpi_accounts.clone(),
            None,
        )
    }

    pub fn place_order(
        ctx: Context<PlaceOrderCaller>,
        price: u64,
        size: u64,
        side: Side,
        client_order_id: Option<u64>,
    ) -> Result<()> {
        zeta_client::place_order(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.place_order_cpi_accounts.clone(),
            None,
            price,
            size,
            side,
            client_order_id,
        )
    }

    pub fn place_order_v3(
        ctx: Context<PlaceOrderCaller>,
        price: u64,
        size: u64,
        side: Side,
        order_type: OrderType,
        client_order_id: Option<u64>,
        tag: Option<String>,
    ) -> Result<()> {
        zeta_client::place_order_v3(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.place_order_cpi_accounts.clone(),
            None,
            price,
            size,
            side,
            order_type,
            client_order_id,
            tag,
        )
    }

    pub fn cancel_order(ctx: Context<CancelOrderCaller>, side: Side, order_id: u128) -> Result<()> {
        zeta_client::cancel_order(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.cancel_order_cpi_accounts.clone(),
            None,
            side,
            order_id,
        )
    }

    pub fn cancel_all_market_orders(ctx: Context<CancelOrderCaller>) -> Result<()> {
        zeta_client::cancel_all_market_orders(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.cancel_order_cpi_accounts.clone(),
            None,
        )
    }

    pub fn read_program_data(ctx: Context<ReadProgramData>) -> Result<()> {
        let zeta_group =
            deserialize_account_info_zerocopy::<ZetaGroup>(&ctx.accounts.zeta_group).unwrap();

        // Get the data for the front expiration.
        let expiry_index = zeta_group.front_expiry_index as usize;
        let expiry_series = zeta_group.expiry_series[expiry_index];
        {
            // The unix timestamp that the products are tradeable.
            msg!("Active timestamp {}", expiry_series.active_ts);

            // The unix timestamp that the products expire.
            msg!("Expiry timestamp {}", expiry_series.expiry_ts);

            let status = zeta_group.expiry_series[expiry_index].status()?;

            // If the market is tradeable.
            msg!("Is market live?: {:?}", status == ExpirySeriesStatus::Live);

            // If the market has expired.
            msg!(
                "Is market expired?: {:?}",
                status == ExpirySeriesStatus::Expired
            );
        }

        // Show the data for all products in a given expiry series. Use the front expiry.
        let products_slice = zeta_group.get_products_slice(expiry_index);
        for i in 0..products_slice.len() {
            let product = &products_slice[i];

            // The market index of the given product.
            // This allows for direct indexing into zeta_group.
            // i.e `product == &zeta_group.products[product_index]`
            let market_index = get_products_slice_market_index(expiry_index, i);

            // Strike has 6 decimals of precision.
            let strike = product.strike.get_strike()?;

            // The serum market this product trades on.
            let _market = product.market;

            // Call / Put / Future
            let kind = product.kind;

            msg!(&format!(
                "Market index = {}, Strike = {}, Kind = {:?}",
                market_index, strike, kind
            ));
        }

        // This returns the oracle price as a fixed point integer with 6 decimals of precision
        let oracle_price = get_native_oracle_price(&ctx.accounts.oracle);
        msg!("Oracle price {:?}", oracle_price);

        // Get the mark price and greek data for the first product in the expiry series.
        // This happens to be the lowest strike call.
        let product_index = 0;
        let market_index = get_products_slice_market_index(expiry_index, product_index);

        let greeks = deserialize_account_info_zerocopy::<Greeks>(&ctx.accounts.greeks).unwrap();
        let market_mark_prices = greeks.get_mark_prices_slice(expiry_index)[product_index];
        let market_product_greeks = greeks.get_product_greeks_slice(expiry_index)[product_index];

        msg!(&format!(
            "Market index = {}, Mark price = {}, Delta = {}, Vega = {:?}, IV = {:?}",
            market_index,
            market_mark_prices,
            market_product_greeks.delta,
            Decimal::from(market_product_greeks.vega),
            Decimal::from(market_product_greeks.volatility)
        ));

        let margin_account =
            deserialize_account_info_zerocopy::<MarginAccount>(&ctx.accounts.margin_account)
                .unwrap();

        // Position details for a given market index.
        let size = margin_account.product_ledgers[market_index].position.size;
        let cost_of_trades = margin_account.product_ledgers[market_index]
            .position
            .cost_of_trades;

        msg!(
            "Margin account position for market index {}: Size={}, Cost of trades={}",
            market_index,
            size,
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

        let margin_account_state = calculate_margin_account_state(
            &zeta_group,
            &margin_account,
            &greeks,
            &ctx.accounts.oracle,
        );

        msg!("Margin account state: {:?}", margin_account_state);

        Ok(())
    }

    pub fn position_movement(
        ctx: Context<PositionMovementCaller>,
        movement_type: MovementType,
        movements: Vec<PositionMovementArg>,
    ) -> Result<()> {
        zeta_client::position_movement(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.position_movement_cpi_accounts.clone(),
            None,
            movement_type,
            movements,
        )
    }

    pub fn transfer_excess_spread_balance(
        ctx: Context<TransferExcessSpreadBalanceCaller>,
    ) -> Result<()> {
        zeta_client::transfer_excess_spread_balance(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts
                .transfer_excess_spread_balance_cpi_accounts
                .clone(),
            None,
        )
    }
}

#[error_code]
pub enum FuzeErrorCode {
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
    #[msg("Invalid option kind, must be Call or Put")]
    InvalidOptionKind,
}
