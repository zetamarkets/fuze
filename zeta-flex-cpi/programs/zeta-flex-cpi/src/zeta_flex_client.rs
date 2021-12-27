use crate::zeta_flex_constants::*;
use crate::zeta_flex_context::*;
use crate::*;
use cpi_interface::global_interface;

/// Zeta Program Client
/// Defines a clean interface and set of helper functions to make CPI calls to the Zeta Program

#[global_interface]
pub trait ZetaInterface<'info, T: Accounts<'info>> {
    fn initialize_auction(ctx: Context<T>, args: InitializeAuctionArgs) -> ProgramResult;
}

pub fn initialize_auction<'info>(
    zeta_program: AccountInfo<'info>,
    cpi_accounts: InitializeAuction<'info>,
    args: InitializeAuctionArgs,
) -> ProgramResult {
    // let (_pda, nonce) = Pubkey::find_program_address(
    //     &[
    //         // cpi_accounts.zeta_group.key.as_ref(),
    //         // cpi_accounts.authority.key.as_ref(),
    //     ],
    //     &zeta_program.key.clone(),
    // );
    let cpi_ctx = CpiContext::new(zeta_program, cpi_accounts);
    zeta_interface::initialize_auction(cpi_ctx, args)
}
