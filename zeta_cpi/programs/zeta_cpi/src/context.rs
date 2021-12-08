use crate::zeta_context::*;
use crate::*;

// CPI Program Context
// Edit this as you wish for your own program instructions

#[derive(Accounts)]
pub struct InitializeMarginAccountCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub initialize_margin_cpi_accounts: InitializeMarginAccount<'info>,
}

#[derive(Accounts)]
pub struct DepositCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub deposit_cpi_accounts: Deposit<'info>,
}

#[derive(Accounts)]
pub struct WithdrawCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub withdraw_cpi_accounts: Withdraw<'info>,
}

#[derive(Accounts)]
pub struct InitializeOpenOrdersCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub initialize_open_orders_cpi_accounts: InitializeOpenOrders<'info>,
}

#[derive(Accounts)]
pub struct PlaceOrderCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub place_order_cpi_accounts: PlaceOrder<'info>,
}

#[derive(Accounts)]
pub struct CancelOrderCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub cancel_order_cpi_accounts: CancelOrder<'info>,
}

#[derive(Accounts)]
pub struct ReadProgramData<'info> {
    pub state: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    pub margin_account: AccountInfo<'info>,
    pub greeks: AccountInfo<'info>,
    pub oracle: AccountInfo<'info>,
}
