use anchor_lang::prelude::*;

pub mod context;
pub mod pyth_client;
pub mod zeta_flex_account;
pub mod zeta_flex_client;
pub mod zeta_flex_constants;
pub mod zeta_flex_context;
pub mod zeta_flex_utils;
use crate::context::*;
use crate::zeta_flex_account::*;
use crate::zeta_flex_constants::*;
use crate::zeta_flex_context::*;
use crate::zeta_flex_utils::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod zeta_flex_cpi {
    use super::*;

    pub fn initialize_auction(
        ctx: Context<InitializeAuctionCaller>,
        args: InitializeAuctionArgs,
    ) -> ProgramResult {
        zeta_flex_client::initialize_auction(
            ctx.accounts.zeta_flex_program.clone(),
            ctx.accounts.initialize_auction_cpi_accounts.clone(),
            args,
        )
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[error]
pub enum ErrorCode {
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
    #[msg("Token account mint mismatch")]
    TokenAccountMintMismatch,
    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Insufficient options to burn")]
    InsufficientOptionsToBurn,
    #[msg("Only creator can close option account")]
    OnlyCreatorCanCloseOptionAccount,
    #[msg("Only creator can burn options")]
    OnlyCreatorCanBurnOptions,
    #[msg("Option expiration must be in the future")]
    OptionExpirationMustBeInTheFuture,
    #[msg("Option has not expired yet")]
    OptionHasNotExpiredYet,
    #[msg("Only admin can override settlement price")]
    OnlyAdminCanOverrideSettlementPrice,
    #[msg("Before set settlement price time")]
    BeforeSetSettlementPriceTime,
    #[msg("After set settlement price time")]
    AfterSetSettlementPriceTime,
    #[msg("Settlement price already set")]
    SettlementPriceAlreadySet,
    #[msg("SettlementPriceNotSet")]
    SettlementPriceNotSet,
    #[msg("InvalidOracle")]
    InvalidOracle,
    #[msg("Invalid settlment option mint")]
    InvalidSettlementOptionMint,
    #[msg("Cannot burn options after expiry")]
    CannotBurnOptionsAfterExpiry,
    #[msg("Cannot burn options after settlement price is set")]
    CannotBurnOptionsAfterSettlementPriceIsSet,
    #[msg("Owner mismatch")]
    OwnerMismatch,
    #[msg("Option mint mismatch")]
    OptionMintMismatch,
    #[msg("Auction cooldown end has to be after bid end")]
    AuctionCooldownEndHasToBeAfterBidEnd,
    #[msg("Auction bid end has to be in the future")]
    AuctionBidEndHasToBeInTheFuture,
    #[msg("Only creator can cancel auction")]
    OnlyCreatorCanCancelAuction,
    #[msg("Cannot cancel auction after completion")]
    CannotCancelAuctionAfterCompletion,
    #[msg("Cannot bid after bid end")]
    CannotBidAfterBidEnd,
    #[msg("Only bidder can cancel bid")]
    OnlyBidderCanCancelBid,
    #[msg("BidCurrencyMintMismatch")]
    BidCurrencyMintMismatch,
    #[msg("Cannot cancel bid during cooldown period")]
    CannotCancelBidDuringCooldownPeriod,
    #[msg("ExpectedAmountMismatch")]
    ExpectedAmountMismatch,
    #[msg("Cannot accept bid outside of cooldown period")]
    CannotAcceptBidOutsideOfCooldownPeriod,
    #[msg("Invalid bidder auction token account")]
    InvalidBidderAuctionTokenAccount,
    #[msg("Invalid creator bid currency token account")]
    InvalidCreatorBidCurrencyTokenAccount,
    #[msg("Creator native token account should be empty")]
    CreatorNativeTokenAccountShouldBeEmpty,
    #[msg("Native token auction unsupported")]
    NativeTokenAuctionUnsupported,
    #[msg("Cancel bid native token account should be empty")]
    CancelBidNativeTokenAccountShouldBeEmpty,
}
