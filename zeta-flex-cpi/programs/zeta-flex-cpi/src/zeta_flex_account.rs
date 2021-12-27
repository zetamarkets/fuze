use crate::*;

#[account]
#[derive(Default)]
pub struct OptionAccount {
    pub option_account_nonce: u8,
    pub option_mint_nonce: u8,
    pub creator_option_token_account_nonce: u8,
    pub vault_nonce: u8,

    pub underlying_count: u64,
    pub option_mint: Pubkey,
    pub underlying_mint: Pubkey,
    pub creator: Pubkey,
    pub strike: u64,
    pub expiry: u64,
    pub settlement_price: u64,

    pub profit_per_option: u64,
    pub remaining_collateral: u64,
}

#[account]
#[derive(Default)]
pub struct Underlying {
    pub underlying_nonce: u8,
    pub mint: Pubkey,
    pub oracle: Pubkey,
    pub count: u64,
}

#[account]
#[derive(Default)]
pub struct State {
    pub state_nonce: u8,
    pub mint_auth_nonce: u8,
    pub vault_auth_nonce: u8,
    pub admin: Pubkey,
    pub settlement_price_threshold_seconds: u32,
}

#[account]
#[derive(Default)]
pub struct AuctionAccount {
    pub auction_nonce: u8,
    pub auction_vault_nonce: u8,
    pub creator: Pubkey,
    pub creator_bid_currency_token_account: Pubkey,
    pub amount: u64,
    pub bid_end: u64,
    pub cooldown_end: u64,
    pub auction_token_mint: Pubkey,
    pub bid_currency_mint: Pubkey,
}

#[account]
#[derive(Default)]
pub struct BidAccount {
    pub bid_account_nonce: u8,
    pub bid_vault_nonce: u8,
    pub bidder: Pubkey,
    pub amount: u64,
    pub bid_end: u64,
    pub cooldown_end: u64,
    pub auction_account: Pubkey,
    pub bid_vault: Pubkey,
    pub bidder_auction_token_account: Pubkey,
}