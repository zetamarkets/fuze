use crate::*;
use cpi_interface::global_interface;
use crate::zeta_context::*;
use crate::zeta_constants::*;

/// Zeta Program Client
/// Defines a clean interface and set of helper functions to make CPI calls to the Zeta Program

#[global_interface]
pub trait ZetaInterface<'info, T: Accounts<'info>> {
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
    fn cancel_order(
        ctx: Context<T>, 
        side: Side, 
        order_id: u128
    ) -> ProgramResult;
}

pub fn initialize_margin_account<'info>(zeta_program: AccountInfo<'info>, cpi_accounts: InitializeMarginAccount<'info>) -> ProgramResult {
    let (_pda, nonce)  = Pubkey::find_program_address(&[MARGIN_SEED.as_ref(), cpi_accounts.zeta_group.key.as_ref(), cpi_accounts.authority.key.as_ref()], &zeta_program.key.clone());
    let cpi_ctx = CpiContext::new(zeta_program, cpi_accounts);
    zeta_interface::initialize_margin_account(cpi_ctx, nonce)
}

pub fn deposit<'info>(zeta_program: AccountInfo<'info>, cpi_accounts: Deposit<'info>, amount: u64) -> ProgramResult {
    let cpi_ctx = CpiContext::new(zeta_program, cpi_accounts);
    zeta_interface::deposit(cpi_ctx, amount)
}

pub fn withdraw<'info>(zeta_program: AccountInfo<'info>, cpi_accounts: Withdraw<'info>, amount: u64) -> ProgramResult {
    let cpi_ctx = CpiContext::new(zeta_program, cpi_accounts);
    zeta_interface::withdraw(cpi_ctx, amount)
}

pub fn initialize_open_orders<'info>(zeta_program: AccountInfo<'info>, cpi_accounts: InitializeOpenOrders<'info>) -> ProgramResult {
    let (_, nonce)  = Pubkey::find_program_address(&[OPEN_ORDERS_SEED.as_bytes(), cpi_accounts.dex_program.key.as_ref(), cpi_accounts.market.key.as_ref(), cpi_accounts.authority.key.as_ref()], &zeta_program.key.clone());
    let (_, map_nonce)  = Pubkey::find_program_address(&[cpi_accounts.open_orders.key.as_ref()], &zeta_program.key.clone());
    let cpi_ctx = CpiContext::new(zeta_program, cpi_accounts);
    zeta_interface::initialize_open_orders(cpi_ctx, nonce, map_nonce)
}

pub fn place_order<'info>(
    zeta_program: AccountInfo<'info>, 
    cpi_accounts: PlaceOrder<'info>,     
    price: u64,
    size: u32,
    side: Side
) -> ProgramResult {
    let cpi_ctx = CpiContext::new(zeta_program, cpi_accounts);
    zeta_interface::place_order(cpi_ctx, price, size, side)
}

pub fn cancel_order<'info>(
    zeta_program: AccountInfo<'info>, 
    cpi_accounts: CancelOrder<'info>,     
    side: Side, 
    order_id: u128
) -> ProgramResult {
    let cpi_ctx = CpiContext::new(zeta_program, cpi_accounts);
    zeta_interface::cancel_order(cpi_ctx, side, order_id)
}