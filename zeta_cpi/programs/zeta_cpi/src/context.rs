use crate::*;
use anchor_spl::token::Token;
use crate::zeta_client::context::{CreateMarginAccount, InitializeMarginAccount, Deposit, Withdraw, InitializeOpenOrders, PlaceOrder};

// CPI Program Context

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

impl<'info> From<&CreateMarginAccountCaller<'info>> for CreateMarginAccount<'info> {
    fn from(accounts: &CreateMarginAccountCaller<'info>) -> CreateMarginAccount<'info> {
        CreateMarginAccount {
            margin_account: accounts.margin_account.clone(),
            authority: accounts.authority.clone(),
            system_program: accounts.system_program.clone(),
            zeta_program: accounts.zeta_program.clone(),
            zeta_group: accounts.zeta_group.clone(),
        }
    }
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

impl<'info> From<&InitializeMarginAccountCaller<'info>> for InitializeMarginAccount<'info> {
    fn from(accounts: &InitializeMarginAccountCaller<'info>) -> InitializeMarginAccount<'info> {
        InitializeMarginAccount {
            zeta_group: accounts.zeta_group.clone(),
            margin_account: accounts.margin_account.clone(),
            authority: accounts.authority.clone(),
            system_program: accounts.system_program.clone(),
        }
    }
}

#[derive(Accounts)]
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

impl<'info> From<&DepositCaller<'info>> for Deposit<'info> {
    fn from(accounts: &DepositCaller<'info>) -> Deposit<'info> {
        Deposit {
            state: accounts.state.clone(),
            zeta_group: accounts.zeta_group.clone(),
            margin_account: accounts.margin_account.clone(),
            vault: accounts.vault.clone(),
            user_token_account: accounts.user_token_account.clone(),
            authority: accounts.authority.clone(),
            token_program: accounts.token_program.clone(),
        }
    }
}

#[derive(Accounts)]
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

impl<'info> From<&WithdrawCaller<'info>> for Withdraw<'info> {
    fn from(accounts: &WithdrawCaller<'info>) -> Withdraw<'info> {
        Withdraw {
            zeta_group: accounts.zeta_group.clone(),
            state: accounts.state.clone(),
            vault: accounts.vault.clone(),
            margin_account: accounts.margin_account.clone(),
            user_token_account: accounts.user_token_account.clone(),
            token_program: accounts.token_program.clone(),
            authority: accounts.authority.clone(),
            greeks: accounts.greeks.clone(),
            oracle: accounts.oracle.clone(),
        }
    }
}

#[derive(Accounts)]
pub struct InitializeOpenOrdersCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub state: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    pub dex_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub market: AccountInfo<'info>,
    pub serum_authority: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders_map: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> From<&InitializeOpenOrdersCaller<'info>> for InitializeOpenOrders<'info> {
    fn from(accounts: &InitializeOpenOrdersCaller<'info>) -> InitializeOpenOrders<'info> {
        InitializeOpenOrders {
            state: accounts.state.clone(),
            zeta_group: accounts.zeta_group.clone(),
            dex_program: accounts.dex_program.clone(),
            system_program: accounts.system_program.clone(),
            open_orders: accounts.open_orders.clone(),
            margin_account: accounts.margin_account.clone(),
            authority: accounts.authority.clone(),
            market: accounts.market.clone(),
            serum_authority: accounts.serum_authority.clone(),
            open_orders_map: accounts.open_orders_map.clone(),
            rent: accounts.rent.clone(),
        }
    }
}

#[derive(Accounts, Clone)]
pub struct MarketAccounts<'info> {
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    // The `spl_token::Account` that funds will be taken from, i.e., transferred
    // from the user into the market's vault.
    //
    // For bids, this is the base currency. For asks, the quote.
    // This has to be owned by serum_authority PDA as serum checks that the owner
    // of open orders also owns this token account
    #[account(mut)]
    pub order_payer_token_account: AccountInfo<'info>,
    // Also known as the "base" currency. For a given A/B market,
    // this is the vault for the A mint.
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    // Also known as the "quote" currency. For a given A/B market,
    // this is the vault for the B mint.
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    // User wallets, used for settling.
    #[account(mut)]
    pub coin_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub pc_wallet: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct PlaceOrderCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub state: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub dex_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub serum_authority: AccountInfo<'info>,
    #[account(mut)]
    pub greeks: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub market_accounts: MarketAccounts<'info>,
    pub oracle: AccountInfo<'info>,
}

impl<'info> From<&PlaceOrderCaller<'info>> for PlaceOrder<'info> {
    fn from(accounts: &PlaceOrderCaller<'info>) -> PlaceOrder<'info> {
        PlaceOrder {
            state: accounts.state.clone(),
            zeta_group: accounts.zeta_group.clone(),
            margin_account: accounts.margin_account.clone(),
            authority: accounts.authority.clone(),
            dex_program: accounts.dex_program.clone(),
            token_program: accounts.token_program.clone(),
            serum_authority: accounts.serum_authority.clone(),
            greeks: accounts.greeks.clone(),
            open_orders: accounts.open_orders.clone(),
            rent: accounts.rent.clone(),
            market_accounts: accounts.market_accounts.clone(),
            oracle: accounts.oracle.clone(),
        }
    }
}