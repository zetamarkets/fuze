use crate::constants::*;
use crate::zeta_context::*;
use crate::*;

#[derive(Accounts)]
#[instruction(vault_name: String, vault_lamports: u64, bumps: VaultBumps)]
pub struct InitializeVault<'info> {
    // vault Authority accounts
    #[account(mut)]
    pub vault_admin: Signer<'info>,
    // vault Accounts
    #[account(
        init,
        seeds = [vault_name.as_bytes()],
        bump = bumps.vault,
        payer = vault_admin
    )]
    pub vault: Box<Account<'info, Vault>>,
    // This is the PDA that holds SOL to pay for the margin account
    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes(), vault_name.as_bytes()],
        bump = bumps.vault_authority
    )]
    pub vault_authority: AccountInfo<'info>,
    #[account(address = address::usdc::ID)]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        mint::decimals = PLATFORM_PRECISION as u8,
        mint::authority = vault_authority,
        seeds = [REDEEMABLE_MINT_SEED.as_bytes(), vault_name.as_bytes()],
        bump = bumps.redeemable_mint,
        payer = vault_admin
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        token::mint = usdc_mint,
        token::authority = vault_authority,
        seeds = [VAULT_USDC_SEED.as_bytes(), vault_name.as_bytes()],
        bump = bumps.vault_usdc,
        payer = vault_admin
    )]
    pub vault_usdc: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitUserRedeemableTokenAccount<'info> {
    // User Accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(
        init,
        token::mint = redeemable_mint,
        token::authority = user_authority,
        seeds = [USER_REDEEMABLE_SEED.as_bytes(),
            vault.vault_name.as_ref().strip(),
            user_authority.key().as_ref()],
        bump,
        payer = user_authority
    )]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault)]
    pub vault: Box<Account<'info, Vault>>,
    #[account(seeds = [VAULT_AUTHORITY_SEED.as_bytes(), vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault_authority)]
    pub vault_authority: AccountInfo<'info>,
    #[account(seeds = [REDEEMABLE_MINT_SEED.as_bytes(), vault.vault_name.as_ref().strip()],
        bump = vault.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct DepositVault<'info> {
    // User Accounts
    pub user_authority: Signer<'info>,
    #[account(
        mut,
        constraint = user_usdc.owner == user_authority.key() @ ErrorCode::InvalidUserUsdcAccountOwner,
        constraint = user_usdc.mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub user_usdc: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [USER_REDEEMABLE_SEED.as_bytes(),
            vault.vault_name.as_ref().strip(),
            user_authority.key().as_ref()],
        bump = bump
    )]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(
        seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        constraint = vault.usdc_mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub vault: Box<Account<'info, Vault>>,
    #[account(
        seeds = [VAULT_AUTHORITY_SEED.as_bytes(), vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault_authority
    )]
    pub vault_authority: AccountInfo<'info>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [REDEEMABLE_MINT_SEED.as_bytes(), vault.vault_name.as_ref().strip()],
        bump = vault.bumps.redeemable_mint
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [VAULT_USDC_SEED.as_bytes(), vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault_usdc
    )]
    pub vault_usdc: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct WithdrawVault<'info> {
    // User Accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(
        mut,
        constraint = user_usdc.owner == user_authority.key() @ ErrorCode::InvalidUserUsdcAccountOwner,
        constraint = user_usdc.mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub user_usdc: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [USER_REDEEMABLE_SEED.as_bytes(),
            vault.vault_name.as_ref().strip(),
            user_authority.key().as_ref()],
        bump = bump
    )]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(
        seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        constraint = vault.usdc_mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub vault: Box<Account<'info, Vault>>,
    #[account(
        seeds = [VAULT_AUTHORITY_SEED.as_bytes(), vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault_authority
    )]
    pub vault_authority: AccountInfo<'info>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [REDEEMABLE_MINT_SEED.as_bytes(), vault.vault_name.as_ref().strip()],
        bump = vault.bumps.redeemable_mint
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [VAULT_USDC_SEED.as_bytes(), vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault_usdc
    )]
    pub vault_usdc: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeZetaMarginAccount<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_admin: Signer<'info>,
    #[account(
        seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        constraint = vault.vault_admin == vault_admin.key() @ ErrorCode::InvalidVaultAdmin,
        constraint = vault.usdc_mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub initialize_margin_cpi_accounts: InitializeMarginAccount<'info>,
}

#[derive(Accounts)]
pub struct DepositZeta<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_admin: Signer<'info>,
    #[account(
        seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        constraint = vault.vault_admin == vault_admin.key() @ ErrorCode::InvalidVaultAdmin,
        constraint = vault.usdc_mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub deposit_cpi_accounts: Deposit<'info>,
}

#[derive(Accounts)]
pub struct WithdrawZeta<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_admin: Signer<'info>,
    #[account(
        seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        constraint = vault.vault_admin == vault_admin.key() @ ErrorCode::InvalidVaultAdmin,
        constraint = vault.usdc_mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub withdraw_cpi_accounts: Withdraw<'info>,
}

#[derive(Accounts)]
pub struct InitializeZetaOpenOrders<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_admin: Signer<'info>,
    #[account(
        seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        constraint = vault.vault_admin == vault_admin.key() @ ErrorCode::InvalidVaultAdmin,
        constraint = vault.usdc_mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub initialize_open_orders_cpi_accounts: InitializeOpenOrders<'info>,
}

#[derive(Accounts)]
pub struct PlaceAuctionOrder<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_admin: Signer<'info>,
    #[account(
        seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        constraint = vault.vault_admin == vault_admin.key() @ ErrorCode::InvalidVaultAdmin,
        constraint = vault.usdc_mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub place_order_cpi_accounts: PlaceOrder<'info>,
}

#[derive(Accounts)]
pub struct CancelAuctionOrder<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_admin: Signer<'info>,
    #[account(
        seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        constraint = vault.vault_admin == vault_admin.key() @ ErrorCode::InvalidVaultAdmin,
        constraint = vault.usdc_mint == usdc_mint.key() @ ErrorCode::InvalidUsdcMint
    )]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub cancel_order_cpi_accounts: CancelOrder<'info>,
}

#[derive(Accounts)]
pub struct RolloverVault<'info> {
    // vault Authority accounts
    #[account(mut)]
    pub vault_admin: Signer<'info>,
    // vault Accounts
    #[account(
        mut,
        constraint = vault.vault_admin == vault_admin.key() @ ErrorCode::InvalidVaultAdmin
    )]
    pub vault: Box<Account<'info, Vault>>,
}

#[account]
#[derive(Default)]
pub struct Vault {
    pub vault_name: [u8; 20], // Setting an arbitrary max of twenty characters in the vault name.
    pub bumps: VaultBumps,
    pub vault_admin: Pubkey,

    pub usdc_mint: Pubkey,
    pub redeemable_mint: Pubkey,
    pub vault_usdc: Pubkey,

    pub epoch_times: EpochTimes,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy)]
pub struct EpochTimes {
    pub start_epoch: i64,      // Friday W1 10am UTC
    pub end_deposits: i64,     // Friday W1 11am UTC
    pub start_auction: i64,    // Friday W1 12:00pm UTC
    pub end_auction: i64,      // Friday W1 12:05pm UTC
    pub start_settlement: i64, // Friday W2 8am UTC
    pub end_epoch: i64,        // Friday W2 10am UTC
    pub epoch_cadence: u64,    // spacing between successive epochs in seconds
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct VaultBumps {
    pub vault: u8,
    pub vault_authority: u8,
    pub redeemable_mint: u8,
    pub vault_usdc: u8,
}

// CPI context traits

impl<'info> DepositVault<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.user_usdc.to_account_info(),
            to: self.vault_usdc.to_account_info(),
            authority: self.user_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn into_mint_to_context<'a, 'b, 'c>(
        &self,
        signer: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.redeemable_mint.to_account_info(),
            to: self.user_redeemable.to_account_info(),
            authority: self.vault_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer)
    }
}

impl<'info> WithdrawVault<'info> {
    pub fn into_burn_context<'a, 'b, 'c>(
        &self,
        signer: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: self.redeemable_mint.to_account_info(),
            to: self.user_redeemable.to_account_info(),
            authority: self.user_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer)
    }

    pub fn into_transfer_context<'a, 'b, 'c>(
        &self,
        signer: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault_usdc.to_account_info(),
            to: self.user_usdc.to_account_info(),
            authority: self.vault_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer)
    }

    pub fn into_close_account_context<'a, 'b, 'c>(
        &self,
        signer: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.user_redeemable.to_account_info(),
            destination: self.user_authority.to_account_info(),
            authority: self.vault_authority.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer)
    }
}
