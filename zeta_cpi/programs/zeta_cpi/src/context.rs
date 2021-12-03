use crate::*;

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
pub struct InitializeMarginAccount<'info> {
    pub zeta_group: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
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

#[derive(Accounts)]
pub struct InitializeOpenOrders<'info> {
    pub state: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    pub dex_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    // Marked mutable since it pays
    #[account(mut)]
    pub authority: Signer<'info>,
    pub market: AccountInfo<'info>,
    pub serum_authority: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders_map: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

// Market accounts are the accounts used to place orders against the dex minus
// common accounts, i.e., program ids, sysvars, and the `pc_wallet`.
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
pub struct PlaceOrder<'info> {
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