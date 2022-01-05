use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{self, Burn, CloseAccount, Mint, MintTo, Token, TokenAccount, Transfer};
use rust_decimal::prelude::*;
use std::ops::Deref;

pub mod address;
pub mod constants;
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
use constants::*;

declare_id!("stdcqm7Cc8Bj1JZfBPYY8Hyzqqjabm7ugk6k74QEm1B");

#[program]
pub mod vault {

    use super::*;

    #[access_control(validate_epoch_times(epoch_times))]
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        vault_name: String,
        vault_lamports: u64,
        bumps: VaultBumps,
        epoch_times: EpochTimes,
    ) -> ProgramResult {
        msg!("Initialize vault");

        let vault = &mut ctx.accounts.vault;

        let name_bytes = vault_name.as_bytes();
        let mut name_data = [b' '; 20];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);

        vault.vault_name = name_data;
        vault.bumps = bumps;
        vault.vault_admin = ctx.accounts.vault_admin.key();

        vault.usdc_mint = ctx.accounts.usdc_mint.key();
        vault.redeemable_mint = ctx.accounts.redeemable_mint.key();
        vault.vault_usdc = ctx.accounts.vault_usdc.key();

        vault.epoch_times = epoch_times;

        // Transfer initial lamport balance to vault payer
        msg!(
            "Transferring {} lamports from `vault_admin` to `vault_authority`",
            vault_lamports
        );
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.vault_admin.key(),
                &ctx.accounts.vault_authority.key(),
                vault_lamports,
            ),
            &[
                ctx.accounts.vault_admin.to_account_info(),
                ctx.accounts.vault_authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    #[access_control(deposit_withdraw_phase(&ctx.accounts.vault))]
    pub fn initialize_user_redeemable_token_account(
        ctx: Context<InitUserRedeemableTokenAccount>,
    ) -> ProgramResult {
        msg!("Initialize user redeemable token account");
        Ok(())
    }

    #[access_control(deposit_withdraw_phase(&ctx.accounts.vault))]
    pub fn deposit_vault(ctx: Context<DepositVault>, _bump: u8, usdc_amount: u64) -> ProgramResult {
        msg!("Deposit into vault");
        // While token::transfer will check this, we prefer a verbose err msg.
        if ctx.accounts.user_usdc.amount < usdc_amount {
            return Err(ErrorCode::InsufficientUsdcBalance.into());
        }

        // Transfer user's USDC to vault USDC account.
        // Calculate USDC tokens due based on the redeem:usdc exchange rate P_z = ( N_u / N_z ).
        // n_u = P_z * n_z
        // If N_z == 0, then set P_z = 1
        let mut redeemable_amount = usdc_amount as u128;
        if (ctx.accounts.redeemable_mint.supply > 0) {
            redeemable_amount = (usdc_amount as u128)
                .checked_mul(ctx.accounts.redeemable_mint.supply as u128)
                .unwrap()
                .checked_div(ctx.accounts.vault_usdc.amount as u128)
                .unwrap();
        }
        token::transfer(
            ctx.accounts.into_transfer_context(),
            redeemable_amount as u64,
        )?;

        // Mint Redeemable to user Redeemable account.
        let vault_name = ctx.accounts.vault.vault_name.as_ref();
        let seeds = vault_authority_seeds!(
            vault_name = vault_name,
            bump = ctx.accounts.vault.bumps.vault_authority
        );
        let signer = &[&seeds[..]];
        token::mint_to(ctx.accounts.into_mint_to_context(signer), usdc_amount)?;

        Ok(())
    }

    #[access_control(deposit_withdraw_phase(&ctx.accounts.vault))]
    pub fn withdraw_vault(
        ctx: Context<WithdrawVault>,
        _bump: u8,
        redeemable_amount: u64,
    ) -> ProgramResult {
        msg!("Withdraw from vault");
        // While token::burn will check this, we prefer a verbose err msg.
        if ctx.accounts.user_redeemable.amount < redeemable_amount {
            return Err(ErrorCode::InsufficientRedeemableBalance.into());
        }

        // Calculate USDC tokens due based on the redeem:usdc exchange rate P_z = ( N_u / N_z ).
        // n_u = P_z * n_z
        let usdc_amount = (redeemable_amount as u128)
            .checked_mul(ctx.accounts.vault_usdc.amount as u128)
            .unwrap()
            .checked_div(ctx.accounts.redeemable_mint.supply as u128)
            .unwrap();

        let vault_name = ctx.accounts.vault.vault_name.as_ref();
        let seeds = vault_authority_seeds!(
            vault_name = vault_name,
            bump = ctx.accounts.vault.bumps.vault_authority
        );
        let signer = &[&seeds[..]];

        // Burn the user's redeemable tokens.
        token::burn(ctx.accounts.into_burn_context(signer), redeemable_amount)?;

        // Transfer USDC from vault account to the user's usdc account.
        token::transfer(
            ctx.accounts.into_transfer_context(signer),
            usdc_amount as u64,
        )?;

        // Send rent back to user if account is empty
        ctx.accounts.user_redeemable.reload()?;
        if ctx.accounts.user_redeemable.amount == 0 {
            token::close_account(ctx.accounts.into_close_account_context(signer))?;
        }

        Ok(())
    }

    pub fn initialize_zeta_margin_account(
        ctx: Context<InitializeZetaMarginAccount>,
    ) -> ProgramResult {
        let vault_name = ctx.accounts.vault.vault_name.as_ref();
        let seeds = vault_authority_seeds!(
            vault_name = vault_name,
            bump = ctx.accounts.vault.bumps.vault_authority
        );
        zeta_client::initialize_margin_account(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.initialize_margin_cpi_accounts.clone(),
            seeds,
        )
    }

    pub fn deposit_zeta(ctx: Context<DepositZeta>, amount: u64) -> ProgramResult {
        let vault_name = ctx.accounts.vault.vault_name.as_ref();
        let seeds = vault_authority_seeds!(
            vault_name = vault_name,
            bump = ctx.accounts.vault.bumps.vault_authority
        );
        msg!("Deposit into zeta margin account");
        zeta_client::deposit(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.deposit_cpi_accounts.clone(),
            seeds,
            amount,
        )
    }

    pub fn withdraw_zeta(ctx: Context<WithdrawZeta>, amount: u64) -> ProgramResult {
        let vault_name = ctx.accounts.vault.vault_name.as_ref();
        let seeds = vault_authority_seeds!(
            vault_name = vault_name,
            bump = ctx.accounts.vault.bumps.vault_authority
        );
        msg!("Withdraw from zeta margin account");
        zeta_client::withdraw(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.withdraw_cpi_accounts.clone(),
            seeds,
            amount,
        )
    }

    pub fn initialize_zeta_open_orders(ctx: Context<InitializeZetaOpenOrders>) -> ProgramResult {
        let vault_name = ctx.accounts.vault.vault_name.as_ref();
        let seeds = vault_authority_seeds!(
            vault_name = vault_name,
            bump = ctx.accounts.vault.bumps.vault_authority
        );
        msg!("Initialize zeta open orders account");
        zeta_client::initialize_open_orders(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.initialize_open_orders_cpi_accounts.clone(),
            seeds,
        )
    }

    // TODO: in future move this on-chain
    pub fn place_auction_order(
        ctx: Context<PlaceAuctionOrder>,
        price: u64,
        size: u64,
        side: Side,
        client_order_id: Option<u64>,
    ) -> ProgramResult {
        msg!("PLACE AUCTION ORDER");
        let vault_name = ctx.accounts.vault.vault_name.as_ref();
        let seeds = vault_authority_seeds!(
            vault_name = vault_name,
            bump = ctx.accounts.vault.bumps.vault_authority
        );
        zeta_client::place_order(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.place_order_cpi_accounts.clone(),
            seeds,
            price,
            size,
            side,
            client_order_id,
        )
    }

    pub fn cancel_auction_order(
        ctx: Context<CancelAuctionOrder>,
        side: Side,
        order_id: u128,
    ) -> ProgramResult {
        msg!("CANCEL AUCTION ORDER");
        let vault_name = ctx.accounts.vault.vault_name.as_ref();
        let seeds = vault_authority_seeds!(
            vault_name = vault_name,
            bump = ctx.accounts.vault.bumps.vault_authority
        );
        zeta_client::cancel_order(
            ctx.accounts.zeta_program.clone(),
            ctx.accounts.cancel_order_cpi_accounts.clone(),
            seeds,
            side,
            order_id,
        )
    }

    #[access_control(epoch_over(&ctx.accounts.vault))]
    pub fn rollover_vault(ctx: Context<RolloverVault>) -> ProgramResult {
        msg!("Roll over vault");
        let vault = &mut ctx.accounts.vault;
        vault.epoch_times.start_epoch = vault
            .epoch_times
            .start_epoch
            .checked_add(vault.epoch_times.epoch_cadence as i64)
            .unwrap();
        vault.epoch_times.end_deposits = vault
            .epoch_times
            .end_deposits
            .checked_add(vault.epoch_times.epoch_cadence as i64)
            .unwrap();
        vault.epoch_times.start_auction = vault
            .epoch_times
            .start_auction
            .checked_add(vault.epoch_times.epoch_cadence as i64)
            .unwrap();
        vault.epoch_times.end_auction = vault
            .epoch_times
            .end_auction
            .checked_add(vault.epoch_times.epoch_cadence as i64)
            .unwrap();
        vault.epoch_times.start_settlement = vault
            .epoch_times
            .start_settlement
            .checked_add(vault.epoch_times.epoch_cadence as i64)
            .unwrap();
        vault.epoch_times.end_epoch = vault
            .epoch_times
            .end_epoch
            .checked_add(vault.epoch_times.epoch_cadence as i64)
            .unwrap();
        Ok(())
    }
}

#[macro_export]
macro_rules! vault_authority_seeds {
    (
        vault_name = $vault_name:expr,
        bump = $bump:expr
    ) => {
        &[VAULT_AUTHORITY_SEED.as_bytes(), $vault_name.strip(), &[$bump]]
    };
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
    #[msg("Insufficient USDC balance")]
    InsufficientUsdcBalance,
    #[msg("Insufficient redeemable token balance")]
    InsufficientRedeemableBalance,
    #[msg("USDC total and redeemable total don't match")]
    UsdcNotEqRedeem,
    #[msg("Given nonce is invalid")]
    InvalidNonce,
    #[msg("Invalid USDC mint")]
    InvalidUsdcMint,
    #[msg("Invalid user USDC account owner")]
    InvalidUserUsdcAccountOwner,
    #[msg("Invalid vault admin")]
    InvalidVaultAdmin,
}

// Access control modifiers.

// Asserts the vault starts in the future.
fn validate_epoch_times(epoch_times: EpochTimes) -> ProgramResult {
    let clock = Clock::get()?;
    if epoch_times.start_epoch <= clock.unix_timestamp {
        return Err(ErrorCode::VaultFuture.into());
    }
    msg!("{}", epoch_times.start_epoch < epoch_times.end_deposits);
    if !(epoch_times.start_epoch < epoch_times.end_deposits
        && epoch_times.end_deposits < epoch_times.start_auction
        && epoch_times.start_auction < epoch_times.end_auction
        && epoch_times.end_auction < epoch_times.start_settlement
        && epoch_times.start_settlement < epoch_times.end_epoch
        && epoch_times.epoch_cadence
            >= (epoch_times
                .end_epoch
                .checked_sub(epoch_times.start_epoch)
                .unwrap() as u64))
    {
        return Err(ErrorCode::SeqTimes.into());
    }
    Ok(())
}

// Asserts the vault is still accepting deposits and withdrawals.
fn deposit_withdraw_phase(vault: &Vault) -> ProgramResult {
    let clock = Clock::get()?;
    msg!("{}", clock.unix_timestamp);
    msg!("{}", vault.epoch_times.start_epoch);
    if clock.unix_timestamp <= vault.epoch_times.start_epoch {
        return Err(ErrorCode::StartEpochTime.into());
    } else if clock.unix_timestamp > vault.epoch_times.end_deposits {
        return Err(ErrorCode::EndDepositsTime.into());
    }
    Ok(())
}

// Asserts the vault is ready to hold an auction.
fn auction_phase(vault: &Vault) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault.epoch_times.start_auction {
        return Err(ErrorCode::StartAuctionTime.into());
    } else if clock.unix_timestamp > vault.epoch_times.end_auction {
        return Err(ErrorCode::EndAuctionTime.into());
    }
    Ok(())
}

// Asserts the vault is ready to settle on Zeta DEX.
fn settlement_phase(vault: &Vault) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault.epoch_times.start_settlement {
        return Err(ErrorCode::StartSettlementTime.into());
    } else if clock.unix_timestamp > vault.epoch_times.end_epoch {
        return Err(ErrorCode::EndEpochTime.into());
    }
    Ok(())
}

// Asserts the current vault epoch has ended.
fn epoch_over(vault: &Vault) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault.epoch_times.end_epoch {
        return Err(ErrorCode::EpochNotOver.into());
    }
    Ok(())
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
