use crate::*;

// CPI Program Context
// Edit this as you wish for your own program instructions

#[derive(Accounts)]
pub struct InitializeAuctionCaller<'info> {
    pub zeta_flex_program: AccountInfo<'info>,
    pub initialize_auction_cpi_accounts: InitializeAuction<'info>,
}
