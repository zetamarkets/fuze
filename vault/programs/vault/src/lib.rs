use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, CloseAccount, Mint, MintTo, Token, TokenAccount, Transfer};
use rust_decimal::prelude::*;
use std::ops::Deref;

pub mod context;
pub mod pyth_client;
pub mod zeta_account;
pub mod zeta_client;
pub mod zeta_constants;
pub mod zeta_context;
pub mod zeta_utils;
use crate::context::*;
use crate::zeta_account::*;
use crate::zeta_constants::*;
use crate::zeta_utils::*;

const DECIMALS: u8 = 6;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod vault {
    use std::ops::Mul;

    use super::*;

    #[access_control(validate_epoch_times(epoch_times))]
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        vault_name: String,
        bumps: VaultBumps,
        epoch_times: EpochTimes,
    ) -> ProgramResult {
        msg!("INITIALIZE vault");

        let vault_account = &mut ctx.accounts.vault_account;

        let name_bytes = vault_name.as_bytes();
        let mut name_data = [b' '; 20];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);

        vault_account.vault_name = name_data;
        vault_account.bumps = bumps;
        vault_account.vault_authority = ctx.accounts.vault_authority.key();

        vault_account.usdc_mint = ctx.accounts.usdc_mint.key();
        vault_account.redeemable_mint = ctx.accounts.redeemable_mint.key();
        vault_account.vault_usdc = ctx.accounts.vault_usdc.key();

        vault_account.epoch_times = epoch_times;

        Ok(())
    }

    #[access_control(deposit_withdraw_phase(&ctx.accounts.vault_account))]
    pub fn init_user_redeemable(ctx: Context<InitUserRedeemable>) -> ProgramResult {
        msg!("INIT USER REDEEMABLE");
        Ok(())
    }

    #[access_control(deposit_withdraw_phase(&ctx.accounts.vault_account))]
    pub fn exchange_usdc_for_redeemable(
        ctx: Context<ExchangeUsdcForRedeemable>,
        amount: u64,
    ) -> ProgramResult {
        msg!("EXCHANGE USDC FOR REDEEMABLE");
        // While token::transfer will check this, we prefer a verbose err msg.
        if ctx.accounts.user_usdc.amount < amount {
            return Err(ErrorCode::LowUsdc.into());
        }

        // Transfer user's USDC to vault USDC account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.vault_usdc.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Mint Redeemable to user Redeemable account.
        let vault_name = ctx.accounts.vault_account.vault_name.as_ref();
        let seeds = &[
            vault_name.strip(),
            &[ctx.accounts.vault_account.bumps.vault_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = MintTo {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.vault_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, amount)?;

        Ok(())
    }

    #[access_control(deposit_withdraw_phase(&ctx.accounts.vault_account))]
    pub fn exchange_redeemable_for_usdc(
        ctx: Context<ExchangeRedeemableForUsdc>,
        amount: u64,
    ) -> ProgramResult {
        msg!("EXCHANGE REDEEMABLE FOR USDC");
        // While token::burn will check this, we prefer a verbose err msg.
        if ctx.accounts.user_redeemable.amount < amount {
            return Err(ErrorCode::LowRedeemable.into());
        }

        // Calculate USDC tokens due based on % ownership of redeemable pool.
        // usdc_amount = (redeemable_amount / redeemable_mint ) / vault_usdc_amount
        let usdc_amount = (amount as u128)
            .checked_mul(ctx.accounts.vault_usdc.amount as u128)
            .unwrap()
            .checked_div(ctx.accounts.redeemable_mint.supply as u128)
            .unwrap();

        let vault_name = ctx.accounts.vault_account.vault_name.as_ref();
        let seeds = &[
            vault_name.strip(),
            &[ctx.accounts.vault_account.bumps.vault_account],
        ];
        let signer = &[&seeds[..]];

        // Burn the user's redeemable tokens.
        let cpi_accounts = Burn {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.vault_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::burn(cpi_ctx, amount)?;

        // Transfer USDC from vault account to the user's usdc account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_usdc.to_account_info(),
            to: ctx.accounts.user_usdc.to_account_info(),
            authority: ctx.accounts.vault_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, usdc_amount as u64)?;

        // Send rent back to user if account is empty
        ctx.accounts.user_redeemable.reload()?;
        if ctx.accounts.user_redeemable.amount == 0 {
            let cpi_accounts = CloseAccount {
                account: ctx.accounts.user_redeemable.to_account_info(),
                destination: ctx.accounts.user_authority.to_account_info(),
                authority: ctx.accounts.vault_account.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::close_account(cpi_ctx)?;
        }

        Ok(())
    }

    #[access_control(deposit_withdraw_phase(&ctx.accounts.vault_account))]
    pub fn withdraw_vault_usdc(ctx: Context<WithdrawVaultUsdc>) -> ProgramResult {
        msg!("WITHDRAW vault USDC");
        // Transfer total USDC from vault account to vault_authority account.
        let vault_name = ctx.accounts.vault_account.vault_name.as_ref();
        let seeds = &[
            vault_name.strip(),
            &[ctx.accounts.vault_account.bumps.vault_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_usdc.to_account_info(),
            to: ctx.accounts.vault_authority_usdc.to_account_info(),
            authority: ctx.accounts.vault_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.vault_usdc.amount)?;

        Ok(())
    }

    // TODO
    // 1. Select instrument

    #[access_control(auction_phase(&ctx.accounts.vault_account))]
    pub fn init_auction(
        ctx: Context<InitializeAuction>,
        delta: u8,
    ) -> ProgramResult {
        msg!("INITIALIZE AUCTION");
        assert!(delta >= 0 && delta <= 100);
        // Delta values are stored at high precision for pricing
        let native_delta = (delta as u64).checked_mul(10u64.pow(PRICING_PRECISION)).unwrap();

        // 1. Instrument selection: select the specific delta strike and (nearest expiry?)
        let zeta_group = deserialize_account_info_zerocopy::<ZetaGroup>(&ctx.accounts.zeta_group).unwrap();
        // Get the data for the front expiration.
        let front_expiry_index = zeta_group.front_expiry_index as usize;
        let front_expiry = zeta_group.expiry_series[front_expiry_index].expiry_ts;

        let put_greeks = deserialize_account_info_zerocopy::<Greeks>(&ctx.accounts.greeks).unwrap();
        let min_delta_diff = unsigned_abs_diff(native_delta, greeks.product_greeks[0].delta).unwrap();
        let min_delta_index = 0;
        for (i, g) in greeks.product_greeks.iter().enumerate() {
            let delta_diff = unsigned_abs_diff(native_delta, g.delta).unwrap();
            if delta_diff < min_delta_diff {
                min_delta_index = i;
            }
        }


        Ok(())
    }
}

#[error]
pub enum ErrorCode {
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
    // Vault-specific errors
    #[msg("Epoch must start in the future")]
    VaultFuture,
    #[msg("Epoch times are non-sequential")]
    SeqTimes,
    #[msg("Epoch has not started")]
    StartEpochTime,
    #[msg("Deposits period has ended")]
    EndDepositsTime,
    #[msg("Auction has not started")]
    StartAuctionTime,
    #[msg("Auction period has ended")]
    EndAuctionTime,
    #[msg("Settlement has not started")]
    StartSettlementTime,
    #[msg("Epoch has ended")]
    EndEpochTime,
    #[msg("Epoch has not finished yet")]
    EpochNotOver,
    #[msg("Insufficient USDC")]
    LowUsdc,
    #[msg("Insufficient redeemable tokens")]
    LowRedeemable,
    #[msg("USDC total and redeemable total don't match")]
    UsdcNotEqRedeem,
    #[msg("Given nonce is invalid")]
    InvalidNonce,
}

// Access control modifiers.

// Asserts the vault starts in the future.
fn validate_epoch_times(epoch_times: EpochTimes) -> ProgramResult {
    let clock = Clock::get()?;
    if epoch_times.start_epoch <= clock.unix_timestamp {
        return Err(ErrorCode::VaultFuture.into());
    }
    if !(epoch_times.start_epoch < epoch_times.end_deposits
        && epoch_times.end_deposits < epoch_times.start_auction
        && epoch_times.start_auction < epoch_times.end_auction
        && epoch_times.end_auction < epoch_times.start_settlement
        && epoch_times.start_settlement < epoch_times.end_epoch)
    {
        return Err(ErrorCode::SeqTimes.into());
    }
    Ok(())
}

// Asserts the vault is still accepting deposits and withdrawals.
fn deposit_withdraw_phase(vault_account: &VaultAccount) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault_account.epoch_times.start_epoch {
        return Err(ErrorCode::StartEpochTime.into());
    } else if clock.unix_timestamp > vault_account.epoch_times.end_deposits {
        return Err(ErrorCode::EndDepositsTime.into());
    }
    Ok(())
}

// Asserts the vault is ready to hold an auction.
fn auction_phase(vault_account: &VaultAccount) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault_account.epoch_times.start_auction {
        return Err(ErrorCode::StartAuctionTime.into());
    } else if clock.unix_timestamp > vault_account.epoch_times.end_auction {
        return Err(ErrorCode::EndAuctionTime.into());
    }
    Ok(())
}

// Asserts the vault is ready to settle on Zeta DEX.
fn settlement_phase(vault_account: &VaultAccount) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault_account.epoch_times.start_settlement {
        return Err(ErrorCode::StartSettlementTime.into());
    } else if clock.unix_timestamp > vault_account.epoch_times.end_epoch {
        return Err(ErrorCode::EndEpochTime.into());
    }
    Ok(())
}

// // Asserts the current vault epoch has ended.
// fn epoch_over(vault_account: &VaultAccount) -> ProgramResult {
//     let clock = Clock::get()?;
//     if clock.unix_timestamp <= vault_account.epoch_times.end_epoch {
//         return Err(ErrorCode::VaultNotOver.into());
//     }
//     Ok(())
// }

fn unsigned_abs_diff(a: u64, b: u64) -> Option<u64>{
    if a > b {
        a.checked_sub(b)
    } else {
        b.checked_sub(a)
    }
}

/// Trait to allow trimming ascii whitespace from a &[u8].
pub trait StripAsciiWhitespace {
    /// Trim ascii whitespace (based on `is_ascii_whitespace()`) from the
    /// start and end of a slice.
    fn strip(&self) -> &[u8];
}

impl<T: Deref<Target = [u8]>> StripAsciiWhitespace for T {
    fn strip(&self) -> &[u8] {
        let from = match self.iter().position(|x| !x.is_ascii_whitespace()) {
            Some(i) => i,
            None => return &self[0..0],
        };
        let to = self.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
        &self[from..=to]
    }
}