pub const STATE_SEED: &[u8] = b"state";
pub const GREEKS_SEED: &[u8] = b"greeks";
pub const MARKET_NODE_SEED: &[u8] = b"market-node";
pub const OPEN_ORDERS_SEED: &[u8] = b"open-orders";
pub const VAULT_SEED: &[u8] = b"vault";
pub const SERUM_VAULT_SEED: &[u8] = b"serum-vault";
pub const ZETA_VAULT_SEED: &[u8] = b"zeta-vault";
pub const ZETA_GROUP_SEED: &[u8] = b"zeta-group";
pub const WHITELIST_DEPOSIT_SEED: &[u8] = b"whitelist-deposit";
pub const ZETA_INSURANCE_VAULT_SEED: &[u8] = b"zeta-insurance-vault";
pub const ZETA_TREASURY_WALLET_SEED: &[u8] = b"zeta-treasury-wallet";
pub const ZETA_REFERRALS_REWARDS_WALLET_SEED: &[u8] = b"zeta-referrals-rewards-wallet";
pub const WHITELIST_INSURANCE_SEED: &[u8] = b"whitelist-insurance";
pub const USER_INSURANCE_DEPOSIT_SEED: &[u8] = b"user-insurance-deposit";
pub const WHITELIST_TRADING_FEES_SEED: &[u8] = b"whitelist-trading-fees";
pub const SETTLEMENT_SEED: &[u8] = b"settlement";
pub const MARGIN_SEED: &[u8] = b"margin";
pub const SPREAD_SEED: &[u8] = b"spread";
pub const UNDERLYING_SEED: &[u8] = b"underlying";
pub const SERUM_SEED: &[u8] = b"serum";
pub const MINT_AUTH_SEED: &[u8] = b"mint-auth";
pub const BASE_MINT_SEED: &[u8] = b"base-mint";
pub const QUOTE_MINT_SEED: &[u8] = b"quote-mint";
pub const MARKET_SEED: &[u8] = b"market";
pub const MARKET_INDEXES_SEED: &[u8] = b"market-indexes";
pub const SOCIALIZED_LOSS_SEED: &[u8] = b"socialized-loss";
pub const REFERRER_SEED: &[u8] = b"referrer";
pub const REFERRAL_SEED: &[u8] = b"referral";
pub const REFERRER_ALIAS_SEED: &[u8] = b"referrer-alias";
pub const PERP_SYNC_QUEUE_SEED: &[u8] = b"perp-sync-queue";

pub const PLATFORM_PRECISION: u32 = 6;
pub const HALT_SPOT_PRICE_PRECISION: u32 = 6;
pub const PRICING_PRECISION: u32 = 12;
pub const POSITION_PRECISION: u32 = 3;

pub const PLATFORM_PRECISION_MULTIPLIER: i128 = 10i128.pow(PLATFORM_PRECISION);
pub const HALT_SPOT_PRICE_PRECISION_MULTIPLIER: i128 = 10i128.pow(HALT_SPOT_PRICE_PRECISION);
pub const PRICING_PRECISION_MULTIPLIER: i128 = 10i128.pow(PRICING_PRECISION);
pub const POSITION_PRECISION_MULTIPLIER: i128 = 10i128.pow(POSITION_PRECISION);
pub const PRICING_PLATFORM_DIFF_PRECISION_MULTIPLIER: i128 =
    10i128.pow(PRICING_PRECISION - PLATFORM_PRECISION);

pub const EVENT_CRANK_LIMIT: u16 = 20;
pub const DEFAULT_MINT_LOT_SIZE: u64 = 1;
pub const DISCRIMINATOR_SIZE: usize = 8;
pub const MARK_PRICE_PERCENTAGE: u128 = 100;
pub const PRICE_BAND_MULTIPLE: u64 = 10;

// where [] is a given series:
// [n options + 1 future], [n options + 1 future], ..., [n options + 1 future], 1 perp
pub const NUM_STRIKES: usize = 11;
pub const NUM_PRODUCTS_PER_SERIES: usize = NUM_STRIKES * 2 + 1;
pub const SERIES_FUTURE_INDEX: usize = NUM_PRODUCTS_PER_SERIES - 1;
pub const ACTIVE_EXPIRIES: usize = 2;
pub const TOTAL_EXPIRIES: usize = 5;
pub const ACTIVE_MARKETS: usize = ACTIVE_EXPIRIES * NUM_PRODUCTS_PER_SERIES + 1;
pub const TOTAL_MARKETS: usize = (TOTAL_EXPIRIES + 1) * NUM_PRODUCTS_PER_SERIES;
pub const PERP_INDEX: usize = TOTAL_MARKETS - 1;
pub const MARKET_INDEX_LIMIT: usize = 18;

// Cap retreat to 5% of current volatility.
pub const MAX_VOLATILITY_RETREAT_PERCENT: u64 = 5;
pub const MAX_INTEREST_RATE_RETREAT: i64 = 20_000_000_000; // 2%

pub const VOLATILITY_POINTS: usize = 5;
pub const SECONDS_IN_A_YEAR: u64 = 31_536_000;
pub const SECONDS_IN_A_DAY: u64 = 86_400;

// Margin calculations
pub const NATIVE_PRECISION_DENOMINATOR: u128 = 100_000_000; // 100%
pub const POSITION_PRECISION_DENOMINATOR: u128 = 1_000;
pub const BPS_DENOMINATOR: u64 = 10_000;

pub const MIN_PRICE_TICK_SIZE: u64 = 100;

// Expiry line up with deribit expirations
#[cfg(feature = "epoch-offset")]
pub const EPOCH_OFFSET: u64 = 115_200; // 32 hours to 8AM Friday UTC from 12am Thursday UTC.
#[cfg(not(feature = "epoch-offset"))]
pub const EPOCH_OFFSET: u64 = 0;

pub const EPSILON_ERROR: u64 = 1_000;

pub const BID_ORDERS_INDEX: usize = 0;
pub const ASK_ORDERS_INDEX: usize = 1;

pub const MAX_ORDER_TAG_LENGTH: usize = 4;

pub const ORACLE_MAX_LOOKBACK_SECONDS: u64 = 10;
pub const ORACLE_MAX_LOOKBACK_DISTANT_SECONDS: u64 = 30;
pub const ORACLE_MAX_DIVERGENCE_BPS: u64 = 50;

pub const MAX_POSITION_MOVEMENTS: usize = 10;
pub const MAX_TOTAL_SPREAD_ACCOUNT_CONTRACTS: u64 = 100_000_000;
pub const STRIKE_MULTIPLIER: u64 = 10u64.pow(PLATFORM_PRECISION as u32);
