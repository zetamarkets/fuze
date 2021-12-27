use crate::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

/// Zeta Flex Context
/// Leave this as is, it defines the instruction context for the zeta flex program

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeOptionArgs {
    pub collateral_amount: u64,
    pub option_account_nonce: u8,
    pub option_mint_nonce: u8,
    pub token_account_nonce: u8,
    pub vault_nonce: u8,
    pub expiry: u64,
    pub strike: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeAuctionArgs {
    pub auction_nonce: u8,
    pub auction_vault_nonce: u8,
    pub amount: u64,
    pub bid_end: u64,
    pub cooldown_end: u64,
}

#[derive(Accounts, Clone)]
#[instruction(args: InitializeAuctionArgs)]
pub struct InitializeAuction<'info> {
    pub state: AccountInfo<'info>,
    #[account(mut)]
    pub auction_account: AccountInfo<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub vault_authority: AccountInfo<'info>,
    #[account(mut)]
    pub auction_vault: Account<'info, TokenAccount>,
    pub auction_token_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub creator_auction_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub creator_bid_currency_token_account: Box<Account<'info, TokenAccount>>,
    pub bid_currency_mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

// #[derive(Accounts)]
// pub struct CancelAuction<'info> {
//     pub state: AccountInfo<'info>,
//     #[account(mut)]
//     pub auction_account: AccountInfo<'info>,
//     #[account(mut)]
//     pub creator: Signer<'info>,
//     pub vault_authority: AccountInfo<'info>,
//     #[account(mut)]
//     pub auction_vault: Account<'info, TokenAccount>,
//     pub auction_token_mint: Box<Account<'info, Mint>>,
//     #[account(
//         mut,
//         constraint = creator_auction_token_account.mint == auction_token_mint.key() @ ErrorCode::TokenAccountMintMismatch,
//         constraint = creator_auction_token_account.owner == creator.key() @ ErrorCode::InvalidTokenAccountOwner,
//     )]
//     pub creator_auction_token_account: Box<Account<'info, TokenAccount>>,
//     pub creator_bid_currency_token_account: Box<Account<'info, TokenAccount>>,
//     pub token_program: Program<'info, Token>,
// }

// #[derive(Accounts)]
// #[instruction(expected_amount: u64)]
// pub struct AcceptBid<'info> {
//     pub state: Box<Account<'info, State>>,
//     #[account(
//         mut,
//         seeds = [AUCTION_SEED.as_bytes().as_ref(), auction_account.auction_token_mint.as_ref(), auction_account.creator.as_ref(), &auction_account.amount.to_le_bytes(), &auction_account.bid_end.to_le_bytes(), &auction_account.cooldown_end.to_le_bytes()],
//         bump = auction_account.auction_nonce,
//         close = creator,
//     )]
//     pub auction_account: Box<Account<'info, AuctionAccount>>,
//     #[account(mut)]
//     pub bidder: AccountInfo<'info>,
//     #[account(mut)]
//     pub creator: Signer<'info>,
//     #[account(
//         mut,
//         seeds = [BID_ACCOUNT_SEED.as_bytes().as_ref(), auction_account.key().as_ref(), bidder.key().as_ref()],
//         bump = bid_account.bid_account_nonce,
//         close = bidder,
//         constraint = bid_account.amount == expected_amount @ ErrorCode::ExpectedAmountMismatch,
//     )]
//     pub bid_account: Account<'info, BidAccount>,
//     #[account(
//         seeds = [VAULT_AUTH_SEED.as_bytes().as_ref()],
//         bump = state.vault_auth_nonce,
//     )]
//     pub vault_authority: AccountInfo<'info>,
//     #[account(
//         mut,
//         seeds = [BID_VAULT_SEED.as_bytes().as_ref(), auction_account.key().as_ref(), bidder.key().as_ref()],
//         bump = bid_account.bid_vault_nonce,
//         constraint = bid_vault.amount == expected_amount @ ErrorCode::ExpectedAmountMismatch,
//     )]
//     pub bid_vault: Account<'info, TokenAccount>,
//     #[account(
//         mut,
//         seeds = [AUCTION_VAULT_SEED.as_bytes().as_ref(), auction_account.auction_token_mint.as_ref(), creator.key().as_ref(), &auction_account.amount.to_le_bytes(), &auction_account.bid_end.to_le_bytes(), &auction_account.cooldown_end.to_le_bytes()],
//         bump = auction_account.auction_vault_nonce,
//     )]
//     pub auction_vault: Account<'info, TokenAccount>,
//     #[account(
//         mut,
//         constraint = bidder_auction_token_account.mint == auction_account.auction_token_mint @ ErrorCode::TokenAccountMintMismatch,
//         constraint = bidder_auction_token_account.owner == bidder.key() @ ErrorCode::InvalidTokenAccountOwner,
//         constraint = bidder_auction_token_account.key() == bid_account.bidder_auction_token_account @ ErrorCode::InvalidBidderAuctionTokenAccount,
//     )]
//     pub bidder_auction_token_account: Box<Account<'info, TokenAccount>>,
//     #[account(
//         mut,
//         constraint = creator_bid_currency_token_account.mint == auction_account.bid_currency_mint @ ErrorCode::TokenAccountMintMismatch,
//         constraint = creator_bid_currency_token_account.owner == creator.key() @ ErrorCode::InvalidTokenAccountOwner,
//         constraint = creator_bid_currency_token_account.key() == auction_account.creator_bid_currency_token_account @ ErrorCode::InvalidCreatorBidCurrencyTokenAccount,
//     )]
//     pub creator_bid_currency_token_account: Box<Account<'info, TokenAccount>>,
//     pub token_program: Program<'info, Token>,
// }

// #[derive(Accounts)]
// #[instruction(args: InitializeOptionArgs)]
// pub struct InitializeOption<'info> {
//     pub state: Box<Account<'info, State>>,
//     #[account(
//         mut,
//         seeds = [UNDERLYING_SEED.as_bytes().as_ref(), underlying_mint.key().as_ref()],
//         bump = underlying.underlying_nonce,
//     )]
//     pub underlying: Box<Account<'info, Underlying>>,
//     #[account(
//         init,
//         token::mint = underlying_mint,
//         token::authority = vault_authority,
//         seeds = [VAULT_SEED.as_bytes().as_ref(), option_account.key().as_ref()],
//         bump = args.vault_nonce,
//         payer = creator,
//     )]
//     pub vault: Account<'info, TokenAccount>,
//     #[account(
//         seeds = [VAULT_AUTH_SEED.as_bytes().as_ref()],
//         bump = state.vault_auth_nonce,
//     )]
//     pub vault_authority: AccountInfo<'info>,
//     pub underlying_mint: Box<Account<'info, Mint>>,
//     #[account(
//         mut,
//         constraint = underlying_token_account.mint == underlying_mint.key() @ ErrorCode::TokenAccountMintMismatch,
//         constraint = underlying_token_account.owner == creator.key() @ ErrorCode::InvalidTokenAccountOwner,
//         constraint = underlying_token_account.amount >= args.collateral_amount @ ErrorCode::InsufficientFunds,
//     )]
//     pub underlying_token_account: Box<Account<'info, TokenAccount>>,
//     #[account(mut)]
//     pub creator: Signer<'info>,
//     #[account(
//         init,
//         seeds = [OPTION_ACCOUNT_SEED.as_bytes().as_ref(), underlying.key().as_ref(), &underlying.count.to_le_bytes()],
//         bump = args.option_account_nonce,
//         payer = creator,
//     )]
//     pub option_account: Box<Account<'info, OptionAccount>>,
//     #[account(
//         seeds = [MINT_AUTH_SEED.as_bytes().as_ref()],
//         bump = state.mint_auth_nonce,
//     )]
//     pub mint_authority: AccountInfo<'info>,
//     #[account(
//         init,
//         mint::decimals = OPTION_MINT_DECIMALS,
//         mint::authority = mint_authority,
//         seeds = [OPTION_MINT_SEED.as_bytes().as_ref(), option_account.key().as_ref()],
//         bump = args.option_mint_nonce,
//         payer = creator,
//     )]
//     pub option_mint: Box<Account<'info, Mint>>,
//     #[account(
//         init,
//         token::mint = option_mint,
//         token::authority = creator,
//         seeds = [option_mint.key().as_ref(), creator.key().as_ref()],
//         bump = args.token_account_nonce,
//         payer = creator,
//     )]
//     pub user_option_token_account: Box<Account<'info, TokenAccount>>,
//     pub system_program: Program<'info, System>,
//     pub token_program: Program<'info, Token>,
//     pub rent: Sysvar<'info, Rent>,
// }

// #[derive(Accounts)]
// #[instruction(amount: u64)]
// pub struct BurnOption<'info> {
//     pub state: Box<Account<'info, State>>,
//     #[account(
//         seeds = [UNDERLYING_SEED.as_bytes().as_ref(), underlying_mint.key().as_ref()],
//         bump = underlying.underlying_nonce,
//     )]
//     pub underlying: Box<Account<'info, Underlying>>,
//     #[account(
//         mut,
//         seeds = [VAULT_SEED.as_bytes().as_ref(), option_account.key().as_ref()],
//         bump = option_account.vault_nonce,
//     )]
//     pub vault: Box<Account<'info, TokenAccount>>,
//     pub underlying_mint: Box<Account<'info, Mint>>,
//     #[account(
//         mut,
//         constraint = underlying_token_account.mint == underlying_mint.key() @ ErrorCode::TokenAccountMintMismatch,
//         constraint = underlying_token_account.owner == creator.key() @ ErrorCode::InvalidTokenAccountOwner,
//     )]
//     pub underlying_token_account: Box<Account<'info, TokenAccount>>,
//     #[account(
//         mut,
//         constraint = creator.key() == option_account.creator @ ErrorCode::OnlyCreatorCanBurnOptions
//     )]
//     pub creator: Signer<'info>,
//     #[account(
//         seeds = [OPTION_ACCOUNT_SEED.as_bytes().as_ref(), underlying.key().as_ref(), &option_account.underlying_count.to_le_bytes()],
//         bump = option_account.option_account_nonce,
//     )]
//     pub option_account: Box<Account<'info, OptionAccount>>,
//     #[account(
//         seeds = [MINT_AUTH_SEED.as_bytes().as_ref()],
//         bump = state.mint_auth_nonce,
//     )]
//     pub mint_authority: AccountInfo<'info>,
//     #[account(
//         mut,
//         seeds = [OPTION_MINT_SEED.as_bytes().as_ref(), option_account.key().as_ref()],
//         bump = option_account.option_mint_nonce,
//     )]
//     pub option_mint: Box<Account<'info, Mint>>,
//     #[account(
//         mut,
//         seeds = [option_mint.key().as_ref(), creator.key().as_ref()],
//         bump = option_account.creator_option_token_account_nonce,
//         constraint = user_option_token_account.amount >= amount @ ErrorCode::InsufficientOptionsToBurn,
//     )]
//     pub user_option_token_account: Box<Account<'info, TokenAccount>>,
//     pub token_program: Program<'info, Token>,
//     #[account(
//         seeds = [VAULT_AUTH_SEED.as_bytes().as_ref()],
//         bump = state.vault_auth_nonce,
//     )]
//     pub vault_authority: AccountInfo<'info>,
// }

// #[derive(Accounts)]
// pub struct ExerciseOption<'info> {
//     pub state: Box<Account<'info, State>>,
//     #[account(
//         seeds = [UNDERLYING_SEED.as_bytes().as_ref(), underlying_mint.key().as_ref()],
//         bump = underlying.underlying_nonce,
//     )]
//     pub underlying: Box<Account<'info, Underlying>>,
//     #[account(
//         mut,
//         seeds = [VAULT_SEED.as_bytes().as_ref(), option_account.key().as_ref()],
//         bump = option_account.vault_nonce,
//     )]
//     pub vault: Box<Account<'info, TokenAccount>>,
//     pub underlying_mint: Box<Account<'info, Mint>>,
//     #[account(
//         mut,
//         constraint = underlying_token_account.mint == underlying_mint.key() @ ErrorCode::TokenAccountMintMismatch,
//         constraint = underlying_token_account.owner == authority.key() @ ErrorCode::InvalidTokenAccountOwner,
//     )]
//     pub underlying_token_account: Box<Account<'info, TokenAccount>>,
//     #[account(mut)]
//     pub authority: Signer<'info>,
//     #[account(
//         mut,
//         seeds = [OPTION_ACCOUNT_SEED.as_bytes().as_ref(), underlying.key().as_ref(), &option_account.underlying_count.to_le_bytes()],
//         bump = option_account.option_account_nonce,
//     )]
//     pub option_account: Box<Account<'info, OptionAccount>>,
//     #[account(
//         mut,
//         seeds = [OPTION_MINT_SEED.as_bytes().as_ref(), option_account.key().as_ref()],
//         bump = option_account.option_mint_nonce,
//         constraint = option_mint.key() == option_account.option_mint @ ErrorCode::OptionMintMismatch
//     )]
//     pub option_mint: Box<Account<'info, Mint>>,
//     #[account(
//         mut,
//         constraint = user_option_token_account.owner == authority.key() @ ErrorCode::OwnerMismatch,
//         constraint = user_option_token_account.mint == option_mint.key() @ ErrorCode::TokenAccountMintMismatch,
//     )]
//     pub user_option_token_account: Box<Account<'info, TokenAccount>>,
//     pub token_program: Program<'info, Token>,
//     #[account(
//         seeds = [VAULT_AUTH_SEED.as_bytes().as_ref()],
//         bump = state.vault_auth_nonce,
//     )]
//     pub vault_authority: AccountInfo<'info>,
// }

// #[derive(Accounts)]
// pub struct CollectRemainingCollateral<'info> {
//     pub state: Box<Account<'info, State>>,
//     #[account(
//         seeds = [UNDERLYING_SEED.as_bytes().as_ref(), underlying_mint.key().as_ref()],
//         bump = underlying.underlying_nonce,
//     )]
//     pub underlying: Box<Account<'info, Underlying>>,
//     #[account(
//         mut,
//         seeds = [VAULT_SEED.as_bytes().as_ref(), option_account.key().as_ref()],
//         bump = option_account.vault_nonce,
//     )]
//     pub vault: Box<Account<'info, TokenAccount>>,
//     pub underlying_mint: Box<Account<'info, Mint>>,
//     #[account(
//         mut,
//         constraint = underlying_token_account.mint == underlying_mint.key() @ ErrorCode::TokenAccountMintMismatch,
//         constraint = underlying_token_account.owner == creator.key() @ ErrorCode::InvalidTokenAccountOwner,
//     )]
//     pub underlying_token_account: Box<Account<'info, TokenAccount>>,
//     #[account(
//         mut,
//         constraint = creator.key() == option_account.creator @ ErrorCode::OnlyCreatorCanCloseOptionAccount
//     )]
//     pub creator: Signer<'info>,
//     #[account(
//         mut,
//         seeds = [OPTION_ACCOUNT_SEED.as_bytes().as_ref(), underlying.key().as_ref(), &option_account.underlying_count.to_le_bytes()],
//         bump = option_account.option_account_nonce,
//     )]
//     pub option_account: Box<Account<'info, OptionAccount>>,
//     #[account(
//         mut,
//         seeds = [OPTION_MINT_SEED.as_bytes().as_ref(), option_account.key().as_ref()],
//         bump = option_account.option_mint_nonce,
//     )]
//     pub option_mint: Box<Account<'info, Mint>>,
//     #[account(
//         mut,
//         seeds = [option_mint.key().as_ref(), creator.key().as_ref()],
//         bump = option_account.creator_option_token_account_nonce,
//     )]
//     pub user_option_token_account: Box<Account<'info, TokenAccount>>,
//     pub token_program: Program<'info, Token>,
//     #[account(
//         seeds = [VAULT_AUTH_SEED.as_bytes().as_ref()],
//         bump = state.vault_auth_nonce,
//     )]
//     pub vault_authority: AccountInfo<'info>,
// }