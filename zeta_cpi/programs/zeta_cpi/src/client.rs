use crate::*;

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
    fn initialize_open_orders(
        ctx: Context<T>,
        nonce: u8,
        _map_nonce: u8,
    ) -> ProgramResult;
    fn place_order(
        ctx: Context<T>,
        price: u64,
        size: u32,
        side: Side,
    ) -> ProgramResult;
}

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
    let (_pda, nonce)  = Pubkey::find_program_address(&[MARGIN_SEED.as_ref(), ctx.accounts.zeta_group.key.as_ref(), ctx.accounts.authority.key.as_ref()], &cpi_program.key.clone());
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

pub fn initialize_open_orders(
    ctx: Context<InitializeOpenOrdersCaller>,
) -> ProgramResult {
    let cpi_program = ctx.accounts.zeta_program.clone();
    let cpi_accounts = InitializeOpenOrders {
        state: ctx.accounts.state.clone(),
        zeta_group: ctx.accounts.zeta_group.clone(),
        dex_program: ctx.accounts.dex_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
        open_orders: ctx.accounts.open_orders.clone(),
        margin_account: ctx.accounts.margin_account.clone(),
        authority: ctx.accounts.authority.clone(),
        market: ctx.accounts.market.clone(),
        serum_authority: ctx.accounts.serum_authority.clone(),
        open_orders_map: ctx.accounts.open_orders_map.clone(),
        rent: ctx.accounts.rent.clone(),
    };
    let (_, nonce)  = Pubkey::find_program_address(&[OPEN_ORDERS_SEED.as_bytes(), ctx.accounts.dex_program.key.as_ref(), ctx.accounts.market.key.as_ref(), ctx.accounts.authority.key.as_ref()], &cpi_program.key.clone());
    let (_, map_nonce)  = Pubkey::find_program_address(&[ctx.accounts.open_orders.key.as_ref()], &cpi_program.key.clone());
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    zeta_interface::initialize_open_orders(cpi_ctx, nonce, map_nonce)
}

pub fn place_order(
        ctx: Context<PlaceOrderCaller>,         
        price: u64,
        size: u32,
        side: Side
    ) -> ProgramResult {
    let cpi_program = ctx.accounts.zeta_program.clone();
    let cpi_accounts = PlaceOrder {
        state: ctx.accounts.state.clone(),
        zeta_group: ctx.accounts.zeta_group.clone(),
        margin_account: ctx.accounts.margin_account.clone(),
        authority: ctx.accounts.authority.clone(),
        dex_program: ctx.accounts.dex_program.clone(),
        token_program: ctx.accounts.token_program.clone(),
        serum_authority: ctx.accounts.serum_authority.clone(),
        greeks: ctx.accounts.greeks.clone(),
        open_orders: ctx.accounts.open_orders.clone(),
        rent: ctx.accounts.rent.clone(),
        market_accounts: ctx.accounts.market_accounts.clone(),
        oracle: ctx.accounts.oracle.clone(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    zeta_interface::place_order(cpi_ctx, price, size, side)
}