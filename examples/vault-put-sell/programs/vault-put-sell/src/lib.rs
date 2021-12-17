use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use anchor_spl::token::{self, Burn, CloseAccount, Mint, MintTo, Token, TokenAccount, Transfer};
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
pub mod vault_put_sell {
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

    #[access_control(unrestricted_phase(&ctx.accounts.vault_account))]
    pub fn init_user_redeemable(ctx: Context<InitUserRedeemable>) -> ProgramResult {
        msg!("INIT USER REDEEMABLE");
        Ok(())
    }

    #[access_control(unrestricted_phase(&ctx.accounts.vault_account))]
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
            vault_name.trim_ascii_whitespace(),
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

    #[access_control(withdraw_phase(&ctx.accounts.vault_account))]
    pub fn init_escrow_usdc(ctx: Context<InitEscrowUsdc>) -> ProgramResult {
        msg!("INIT ESCROW USDC");
        Ok(())
    }

    #[access_control(withdraw_phase(&ctx.accounts.vault_account))]
    pub fn exchange_redeemable_for_usdc(
        ctx: Context<ExchangeRedeemableForUsdc>,
        amount: u64,
    ) -> ProgramResult {
        msg!("EXCHANGE REDEEMABLE FOR USDC");
        // While token::burn will check this, we prefer a verbose err msg.
        if ctx.accounts.user_redeemable.amount < amount {
            return Err(ErrorCode::LowRedeemable.into());
        }

        let vault_name = ctx.accounts.vault_account.vault_name.as_ref();
        let seeds = &[
            vault_name.trim_ascii_whitespace(),
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

        // Transfer USDC from vault account to the user's escrow account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_usdc.to_account_info(),
            to: ctx.accounts.escrow_usdc.to_account_info(),
            authority: ctx.accounts.vault_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    #[access_control(epoch_over(&ctx.accounts.vault_account))]
    pub fn exchange_redeemable_for_watermelon(
        ctx: Context<ExchangeRedeemableForWatermelon>,
        amount: u64,
    ) -> ProgramResult {
        msg!("EXCHANGE REDEEMABLE FOR WATERMELON");
        // While token::burn will check this, we prefer a verbose err msg.
        if ctx.accounts.user_redeemable.amount < amount {
            return Err(ErrorCode::LowRedeemable.into());
        }

        // Calculate watermelon tokens due.
        let watermelon_amount = (amount as u128)
            .checked_mul(ctx.accounts.vault_watermelon.amount as u128)
            .unwrap()
            .checked_div(ctx.accounts.redeemable_mint.supply as u128)
            .unwrap();

        let vault_name = ctx.accounts.vault_account.vault_name.as_ref();
        let seeds = &[
            vault_name.trim_ascii_whitespace(),
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

        // Transfer Watermelon from vault account to user.
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_watermelon.to_account_info(),
            to: ctx.accounts.user_watermelon.to_account_info(),
            authority: ctx.accounts.vault_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, watermelon_amount as u64)?;

        // Send rent back to user if account is empty
        ctx.accounts.user_redeemable.reload()?;
        if ctx.accounts.user_redeemable.amount == 0 {
            let cpi_accounts = CloseAccount {
                account: ctx.accounts.user_redeemable.to_account_info(),
                destination: ctx.accounts.user_authority.clone(),
                authority: ctx.accounts.vault_account.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::close_account(cpi_ctx)?;
        }

        Ok(())
    }

    #[access_control(epoch_over(&ctx.accounts.vault_account))]
    pub fn withdraw_vault_usdc(ctx: Context<WithdrawVaultUsdc>) -> ProgramResult {
        msg!("WITHDRAW vault USDC");
        // Transfer total USDC from vault account to vault_authority account.
        let vault_name = ctx.accounts.vault_account.vault_name.as_ref();
        let seeds = &[
            vault_name.trim_ascii_whitespace(),
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

    #[access_control(escrow_over(&ctx.accounts.vault_account))]
    pub fn withdraw_from_escrow(ctx: Context<WithdrawFromEscrow>, amount: u64) -> ProgramResult {
        msg!("WITHDRAW FROM ESCROW");
        // While token::transfer will check this, we prefer a verbose err msg.
        if ctx.accounts.escrow_usdc.amount < amount {
            return Err(ErrorCode::LowUsdc.into());
        }

        let vault_name = ctx.accounts.vault_account.vault_name.as_ref();
        let seeds = &[
            vault_name.trim_ascii_whitespace(),
            &[ctx.accounts.vault_account.bumps.vault_account],
        ];
        let signer = &[&seeds[..]];

        // Transfer USDC from user's escrow account to user's USDC account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow_usdc.to_account_info(),
            to: ctx.accounts.user_usdc.to_account_info(),
            authority: ctx.accounts.vault_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        // Send rent back to user if account is empty
        ctx.accounts.escrow_usdc.reload()?;
        if ctx.accounts.escrow_usdc.amount == 0 {
            let cpi_accounts = CloseAccount {
                account: ctx.accounts.escrow_usdc.to_account_info(),
                destination: ctx.accounts.user_authority.clone(),
                authority: ctx.accounts.vault_account.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::close_account(cpi_ctx)?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(vault_name: String, bumps: VaultBumps)]
pub struct InitializeVault<'info> {
    // vault Authority accounts
    #[account(mut)]
    pub vault_authority: Signer<'info>,
    // vault Accounts
    #[account(init,
        seeds = [vault_name.as_bytes()],
        bump = bumps.vault_account,
        payer = vault_authority)]
    pub vault_account: Box<Account<'info, VaultAccount>>,
    // TODO Confirm USDC mint address on mainnet or leave open as an option for other stables
    #[account(constraint = usdc_mint.decimals == DECIMALS)]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(init,
        mint::decimals = DECIMALS,
        mint::authority = vault_account,
        seeds = [vault_name.as_bytes(), b"redeemable_mint".as_ref()],
        bump = bumps.redeemable_mint,
        payer = vault_authority)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(init,
        token::mint = usdc_mint,
        token::authority = vault_account,
        seeds = [vault_name.as_bytes(), b"vault_usdc"],
        bump = bumps.vault_usdc,
        payer = vault_authority)]
    pub vault_usdc: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitUserRedeemable<'info> {
    // User Accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(init,
        token::mint = redeemable_mint,
        token::authority = vault_account,
        seeds = [user_authority.key().as_ref(),
            vault_account.vault_name.as_ref().trim_ascii_whitespace(),
            b"user_redeemable"],
        bump,
        payer = user_authority)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace()],
        bump = vault_account.bumps.vault_account)]
    pub vault_account: Box<Account<'info, VaultAccount>>,
    #[account(seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace(), b"redeemable_mint"],
        bump = vault_account.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExchangeUsdcForRedeemable<'info> {
    // User Accounts
    pub user_authority: Signer<'info>,
    // TODO replace these with the ATA constraints when possible
    #[account(mut,
        constraint = user_usdc.owner == user_authority.key(),
        constraint = user_usdc.mint == usdc_mint.key())]
    pub user_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        seeds = [user_authority.key().as_ref(),
            vault_account.vault_name.as_ref().trim_ascii_whitespace(),
            b"user_redeemable"],
        bump)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace()],
        bump = vault_account.bumps.vault_account,
        has_one = usdc_mint)]
    pub vault_account: Box<Account<'info, VaultAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace(), b"redeemable_mint"],
        bump = vault_account.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace(), b"vault_usdc"],
        bump = vault_account.bumps.vault_usdc)]
    pub vault_usdc: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitEscrowUsdc<'info> {
    // User Accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(init,
        token::mint = usdc_mint,
        token::authority = vault_account,
        seeds =  [user_authority.key().as_ref(),
            vault_account.vault_name.as_ref().trim_ascii_whitespace(),
            b"escrow_usdc"],
        bump,
        payer = user_authority)]
    pub escrow_usdc: Box<Account<'info, TokenAccount>>,
    #[account(seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace()],
        bump = vault_account.bumps.vault_account,
        has_one = usdc_mint)]
    pub vault_account: Box<Account<'info, VaultAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

// #[derive(Accounts)]
// pub struct ExchangeRedeemableForUsdc<'info> {
//     // User Accounts
//     pub user_authority: Signer<'info>,
//     #[account(mut,
//         seeds = [user_authority.key().as_ref(),
//             vault_account.vault_name.as_ref().trim_ascii_whitespace(),
//             b"escrow_usdc"],
//         bump)]
//     pub escrow_usdc: Box<Account<'info, TokenAccount>>,
//     #[account(mut,
//         seeds = [user_authority.key().as_ref(),
//             vault_account.vault_name.as_ref().trim_ascii_whitespace(),
//             b"user_redeemable"],
//         bump)]
//     pub user_redeemable: Box<Account<'info, TokenAccount>>,
//     // vault Accounts
//     #[account(seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace()],
//         bump = vault_account.bumps.vault_account,
//         has_one = usdc_mint)]
//     pub vault_account: Box<Account<'info, VaultAccount>>,
//     pub usdc_mint: Box<Account<'info, Mint>>,
//     pub watermelon_mint: Box<Account<'info, Mint>>,
//     #[account(mut,
//         seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace(), b"redeemable_mint"],
//         bump = vault_account.bumps.redeemable_mint)]
//     pub redeemable_mint: Box<Account<'info, Mint>>,
//     #[account(mut,
//         seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace(), b"vault_usdc"],
//         bump = vault_account.bumps.vault_usdc)]
//     pub vault_usdc: Box<Account<'info, TokenAccount>>,
//     // Programs and Sysvars
//     pub token_program: Program<'info, Token>,
// }

#[derive(Accounts)]
pub struct ExchangeRedeemableForWatermelon<'info> {
    // User does not have to sign, this allows anyone to redeem on their behalf
    // and prevents forgotten / leftover redeemable tokens in the vault vault.
    pub payer: Signer<'info>,
    // User Accounts
    #[account(mut)] // Sol rent from empty redeemable account is refunded to the user
    pub user_authority: AccountInfo<'info>,
    // TODO replace with ATA constraints
    #[account(mut,
        // seeds = [user_authority.key().as_ref(), token_program.key().as_ref(), watermelon_mint.key().as_ref()]
        constraint = user_watermelon.owner == user_authority.key(),
        constraint = user_watermelon.mint == watermelon_mint.key())]
    pub user_watermelon: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        seeds = [user_authority.key().as_ref(),
            vault_account.vault_name.as_ref().trim_ascii_whitespace(),
            b"user_redeemable"],
        bump)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace()],
        bump = vault_account.bumps.vault_account)]
    pub vault_account: Box<Account<'info, VaultAccount>>,
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace(), b"redeemable_mint"],
        bump = vault_account.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        // seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace(), b"pool_watermelon"],
        // bump = ido_account.bumps.vault_watermelon
    )]
    pub vault_watermelon: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawVaultUsdc<'info> {
    // vault Authority Accounts
    pub vault_authority: Signer<'info>,
    // Doesn't need to be an ATA because it might be a DAO account
    #[account(mut,
        constraint = vault_authority_usdc.owner == vault_authority.key(),
        constraint = vault_authority_usdc.mint == usdc_mint.key())]
    pub vault_authority_usdc: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace()],
        bump = vault_account.bumps.vault_account,
        has_one = vault_authority,
        has_one = usdc_mint)]
    pub vault_account: Box<Account<'info, VaultAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace(), b"vault_usdc"],
        bump = vault_account.bumps.vault_usdc)]
    pub vault_usdc: Box<Account<'info, TokenAccount>>,
    // Program and Sysvars
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawFromEscrow<'info> {
    // User does not have to sign, this allows anyone to redeem on their behalf
    // and prevents forgotten / leftover USDC in the vault.
    pub payer: Signer<'info>,
    // User Accounts
    #[account(mut)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut,
        constraint = user_usdc.owner == user_authority.key(),
        constraint = user_usdc.mint == usdc_mint.key())]
    pub user_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        seeds = [user_authority.key().as_ref(),
            vault_account.vault_name.as_ref().trim_ascii_whitespace(),
            b"escrow_usdc"],
        bump)]
    pub escrow_usdc: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(seeds = [vault_account.vault_name.as_ref().trim_ascii_whitespace()],
        bump = vault_account.bumps.vault_account,
        has_one = usdc_mint)]
    pub vault_account: Box<Account<'info, VaultAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    // Programs and Sysvars
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(Default)]
pub struct VaultAccount {
    pub vault_name: [u8; 20], // Setting an arbitrary max of twenty characters in the vault name.
    pub bumps: VaultBumps,
    pub vault_authority: Pubkey,

    pub usdc_mint: Pubkey,
    pub redeemable_mint: Pubkey,
    pub vault_usdc: Pubkey,

    pub epoch_times: EpochTimes,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy)]
pub struct EpochTimes {
    pub start_epoch: i64, // Friday W1 10am UTC
    pub end_deposits: i64, // Friday W1 11am UTC
    pub start_auction: i64, // Friday W1 12:00pm UTC
    pub end_auction: i64, // Friday W1 12:05pm UTC
    pub end_epoch: i64, // Friday W2 8am UTC
    pub end_settlement: i64, // Friday W2 10am UTC
    // pub end_escrow: i64, // 
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct VaultBumps {
    pub vault_account: u8,
    pub redeemable_mint: u8,
    pub vault_usdc: u8,
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
    // Vault-specific errors
    #[msg("Epoch must start in the future")]
    VaultFuture,
    #[msg("Epoch times are non-sequential")]
    SeqTimes,
    #[msg("Epoch has not started")]
    StartvaultTime,
    #[msg("Deposits period has ended")]
    EndDepositsTime,
    #[msg("Epoch has ended")]
    EndvaultTime,
    #[msg("Epoch has not finished yet")]
    VaultNotOver,
    #[msg("Escrow period has not finished yet")]
    EscrowNotOver,
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
        && epoch_times.end_deposits < epoch_times.end_epoch
        && epoch_times.end_epoch < epoch_times.end_escrow)
    {
        return Err(ErrorCode::SeqTimes.into());
    }
    Ok(())
}

// Asserts the vault is still accepting deposits.
fn unrestricted_phase(vault_account: &VaultAccount) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault_account.epoch_times.start_epoch {
        return Err(ErrorCode::StartvaultTime.into());
    } else if vault_account.epoch_times.end_deposits <= clock.unix_timestamp {
        return Err(ErrorCode::EndDepositsTime.into());
    }
    Ok(())
}

// Asserts the vault has started but not yet finished.
fn withdraw_phase(vault_account: &VaultAccount) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault_account.epoch_times.start_epoch {
        return Err(ErrorCode::StartvaultTime.into());
    } else if vault_account.epoch_times.end_epoch <= clock.unix_timestamp {
        return Err(ErrorCode::EndvaultTime.into());
    }
    Ok(())
}

// Asserts the vault sale period has ended.
fn epoch_over(vault_account: &VaultAccount) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault_account.epoch_times.end_epoch {
        return Err(ErrorCode::VaultNotOver.into());
    }
    Ok(())
}

fn escrow_over(vault_account: &VaultAccount) -> ProgramResult {
    let clock = Clock::get()?;
    if clock.unix_timestamp <= vault_account.epoch_times.end_escrow {
        return Err(ErrorCode::EscrowNotOver.into());
    }
    Ok(())
}

/// Trait to allow trimming ascii whitespace from a &[u8].
pub trait TrimAsciiWhitespace {
    /// Trim ascii whitespace (based on `is_ascii_whitespace()`) from the
    /// start and end of a slice.
    fn trim_ascii_whitespace(&self) -> &[u8];
}

impl<T: Deref<Target = [u8]>> TrimAsciiWhitespace for T {
    fn trim_ascii_whitespace(&self) -> &[u8] {
        let from = match self.iter().position(|x| !x.is_ascii_whitespace()) {
            Some(i) => i,
            None => return &self[0..0],
        };
        let to = self.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
        &self[from..=to]
    }
}
