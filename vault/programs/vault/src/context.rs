use crate::zeta_context::*;
use crate::*;

#[derive(Accounts)]
#[instruction(vault_name: String, bumps: VaultBumps)]
pub struct InitializeVault<'info> {
    // vault Authority accounts
    #[account(mut)]
    pub vault_authority: Signer<'info>,
    // vault Accounts
    #[account(init,
        seeds = [vault_name.as_bytes()],
        bump = bumps.vault,
        payer = vault_authority)]
    pub vault: Box<Account<'info, Vault>>,
    // TODO Confirm USDC mint address on mainnet or leave open as an option for other stables
    #[account(constraint = usdc_mint.decimals == DECIMALS)]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(init,
        mint::decimals = DECIMALS,
        mint::authority = vault,
        seeds = [vault_name.as_bytes(), b"redeemable_mint"],
        bump = bumps.redeemable_mint,
        payer = vault_authority)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(init,
        token::mint = usdc_mint,
        token::authority = vault,
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
        token::authority = vault,
        seeds = [user_authority.key().as_ref(),
            vault.vault_name.as_ref().strip(),
            b"user_redeemable"],
        bump,
        payer = user_authority)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault)]
    pub vault: Box<Account<'info, Vault>>,
    #[account(seeds = [vault.vault_name.as_ref().strip(), b"redeemable_mint"],
        bump = vault.bumps.redeemable_mint)]
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
            vault.vault_name.as_ref().strip(),
            b"user_redeemable"],
        bump)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        has_one = usdc_mint)]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [vault.vault_name.as_ref().strip(), b"redeemable_mint"],
        bump = vault.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [vault.vault_name.as_ref().strip(), b"vault_usdc"],
        bump = vault.bumps.vault_usdc)]
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
        token::authority = vault,
        seeds =  [user_authority.key().as_ref(),
            vault.vault_name.as_ref().strip(),
            b"escrow_usdc"],
        bump,
        payer = user_authority)]
    pub escrow_usdc: Box<Account<'info, TokenAccount>>,
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        has_one = usdc_mint)]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExchangeRedeemableForUsdc<'info> {
    // User Accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(mut,
        constraint = user_usdc.owner == user_authority.key(),
        constraint = user_usdc.mint == usdc_mint.key())]
    pub user_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        seeds = [user_authority.key().as_ref(),
            vault.vault_name.as_ref().strip(),
            b"user_redeemable"],
        bump)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // vault Accounts
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        has_one = usdc_mint)]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [vault.vault_name.as_ref().strip(), b"redeemable_mint"],
        bump = vault.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [vault.vault_name.as_ref().strip(), b"vault_usdc"],
        bump = vault.bumps.vault_usdc)]
    pub vault_usdc: Box<Account<'info, TokenAccount>>,
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
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        has_one = vault_authority,
        has_one = usdc_mint)]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [vault.vault_name.as_ref().strip(), b"vault_usdc"],
        bump = vault.bumps.vault_usdc)]
    pub vault_usdc: Box<Account<'info, TokenAccount>>,
    // Program and Sysvars
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeZetaMarginAccount<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_authority: Signer<'info>,
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        has_one = vault_authority,
        has_one = usdc_mint)]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub initialize_margin_cpi_accounts: InitializeMarginAccount<'info>,
}

#[derive(Accounts)]
pub struct DepositZeta<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_authority: Signer<'info>,
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        has_one = vault_authority,
        has_one = usdc_mint)]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub deposit_cpi_accounts: Deposit<'info>,
}

#[derive(Accounts)]
pub struct WithdrawZeta<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_authority: Signer<'info>,
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        has_one = vault_authority,
        has_one = usdc_mint)]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub withdraw_cpi_accounts: Withdraw<'info>,
}

#[derive(Accounts)]
pub struct InitializeZetaOpenOrders<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_authority: Signer<'info>,
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        has_one = vault_authority,
        has_one = usdc_mint)]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub initialize_open_orders_cpi_accounts: InitializeOpenOrders<'info>,
}

#[derive(Accounts)]
pub struct PlaceAuctionOrder<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub vault_authority: Signer<'info>,
    #[account(seeds = [vault.vault_name.as_ref().strip()],
        bump = vault.bumps.vault,
        has_one = vault_authority,
        has_one = usdc_mint)]
    pub vault: Box<Account<'info, Vault>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub place_order_cpi_accounts: PlaceOrder<'info>,
}

#[account]
#[derive(Default)]
pub struct Vault {
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
    pub start_epoch: i64,      // Friday W1 10am UTC
    pub end_deposits: i64,     // Friday W1 11am UTC
    pub start_auction: i64,    // Friday W1 12:00pm UTC
    pub end_auction: i64,      // Friday W1 12:05pm UTC
    pub start_settlement: i64, // Friday W2 8am UTC
    pub end_epoch: i64,        // Friday W2 10am UTC
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct VaultBumps {
    pub vault: u8,
    pub redeemable_mint: u8,
    pub vault_usdc: u8,
}
