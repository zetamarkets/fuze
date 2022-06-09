use crate::*;

#[derive(Debug)]
pub struct MarginAccountState {
    pub balance: u64,                       // Balance of the margin account.
    pub initial_margin: u64,                // Initial margin requirements of orders and positions.
    pub maintenance_margin: u64,            // Maintenance margin requirements of positions.
    pub unrealized_pnl: i64,                // Unrealized pnl of positions.
    pub available_balance_initial: i64,     // Available balance remaining for trading.
    pub available_balance_maintenance: i64, // Available balance before being at risk of liquidation
    pub account_equity: i64,                // Effective value of account
}

pub fn calculate_margin_account_state(
    zeta_group: &ZetaGroup,
    margin_account: &MarginAccount,
    greeks: &Greeks,
    oracle: &AccountInfo,
) -> MarginAccountState {
    let spot_price = get_native_oracle_price(oracle);
    let initial_margin = margin_account.get_initial_margin(greeks, zeta_group, spot_price);
    let maintenance_margin = margin_account.get_maintenance_margin(greeks, zeta_group, spot_price);
    let unrealized_pnl = margin_account.get_unrealized_pnl(greeks);
    let available_balance_initial = (margin_account.balance as i64)
        .checked_add(unrealized_pnl)
        .unwrap()
        .checked_sub(initial_margin as i64)
        .unwrap();
    let available_balance_maintenance = (margin_account.balance as i64)
        .checked_add(unrealized_pnl)
        .unwrap()
        .checked_sub(maintenance_margin as i64)
        .unwrap();
    let account_equity = (margin_account.balance as i64)
        .checked_add(unrealized_pnl)
        .unwrap();

    MarginAccountState {
        balance: margin_account.balance,
        initial_margin,
        maintenance_margin,
        unrealized_pnl,
        available_balance_initial,
        available_balance_maintenance,
        account_equity,
    }
}
