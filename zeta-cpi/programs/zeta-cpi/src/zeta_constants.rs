pub const STATE_SEED: &str = "state";
pub const GREEKS_SEED: &str = "greeks";
pub const MARKET_NODE_SEED: &str = "market-node";
pub const OPEN_ORDERS_SEED: &str = "open-orders";
pub const VAULT_SEED: &str = "vault";
pub const SERUM_VAULT_SEED: &str = "serum-vault";
pub const ZETA_VAULT_SEED: &str = "zeta-vault";
pub const ZETA_GROUP_SEED: &str = "zeta-group";
pub const ZETA_INSURANCE_VAULT_SEED: &str = "zeta-insurance-vault";
pub const WHITELIST_INSURANCE_SEED: &str = "whitelist-insurance";
pub const USER_INSURANCE_DEPOSIT_SEED: &str = "user-insurance-deposit";
pub const WHITELIST_TRADING_FEES_SEED: &str = "whitelist-trading-fees";
pub const SETTLEMENT_SEED: &str = "settlement";
pub const MARGIN_SEED: &str = "margin";
pub const UNDERLYING_SEED: &str = "underlying";
pub const SERUM_SEED: &str = "serum";
pub const MINT_AUTH_SEED: &str = "mint-auth";
pub const BASE_MINT_SEED: &str = "base-mint";
pub const QUOTE_MINT_SEED: &str = "quote-mint";
pub const MARKET_SEED: &str = "market";
pub const MARKET_INDEXES_SEED: &str = "market-indexes";
pub const SOCIALIZED_LOSS_SEED: &str = "socialized-loss";

pub const PLATFORM_PRECISION: u32 = 6;
pub const HALT_SPOT_PRICE_PRECISION: u32 = 6;
pub const PRICING_PRECISION: u32 = 12;
pub const POSITION_PRECISION: u32 = 3;

pub const EVENT_CRANK_LIMIT: u16 = 25;
pub const DEFAULT_MINT_LOT_SIZE: u64 = 1;
pub const DISCRIMINATOR_SIZE: usize = 8;
pub const MARK_PRICE_PERCENTAGE: u128 = 100;
pub const PRICE_BAND_MULTIPLE: u64 = 10;

pub const NUM_STRIKES: usize = 11;
pub const NUM_PRODUCTS_PER_SERIES: usize = NUM_STRIKES * 2 + 1;

// Last index of series is futures.
pub const SERIES_FUTURE_INDEX: usize = NUM_PRODUCTS_PER_SERIES - 1;
pub const ACTIVE_EXPIRIES: usize = 2;
pub const TOTAL_EXPIRIES: usize = 6;
pub const ACTIVE_MARKETS: usize = ACTIVE_EXPIRIES * NUM_PRODUCTS_PER_SERIES;
pub const TOTAL_MARKETS: usize = TOTAL_EXPIRIES * NUM_PRODUCTS_PER_SERIES;
pub const MARKET_INDEX_LIMIT: usize = 40;

// Pricing.

// Sense check bounds for retreats.
pub const MAX_INTEREST_RATE: i64 = 1_000_000_000_000; // 100%
pub const MIN_INTEREST_RATE: i64 = -1_000_000_000_000; // -100%

pub const MIN_VOLATILITY: u64 = 100_000_000_000; // 10 points.
pub const MAX_VOLATILITY: u64 = 5_000_000_000_000; // 500 points.

// Cap retreat to 5% of current volatility.
pub const MAX_VOLATILITY_RETREAT_PERCENT: u64 = 5;
pub const MAX_INTEREST_RATE_RETREAT: i64 = 20_000_000_000; // 2%

pub const VOLATILITY_POINTS: usize = 5;
pub const SECONDS_IN_A_YEAR: u64 = 31_536_000;

// Margin calculations
pub const NATIVE_PRECISION_DENOMINATOR: u128 = 100_000_000; // 100%
pub const POSITION_PRECISION_DENOMINATOR: u128 = 1_000;

// Futures
pub const FUTURE_MARGIN_INITIAL: u128 = 15_000_000; // 15.0%
pub const FUTURE_MARGIN_MAINTENANCE: u128 = 7_500_000; // 7.5%

// Options initial
pub const OPTION_MARK_PCT_LONG_INITIAL: u128 = 100_000_000;
pub const OPTION_SPOT_PCT_LONG_INITIAL: u128 = 15_000_000;
pub const OPTION_SPOT_PCT_SHORT_INITIAL: u128 = 10_000_000;
pub const OPTION_BASE_PCT_SHORT_INITIAL: u128 = 25_000_000;

// Options maintenance
pub const OPTION_MARK_PCT_LONG_MAINTENANCE: u128 = 100_000_000;
pub const OPTION_SPOT_PCT_LONG_MAINTENANCE: u128 = 7_500_000;
pub const OPTION_SPOT_PCT_SHORT_MAINTENANCE: u128 = 12_500_000;
pub const OPTION_BASE_PCT_SHORT_MAINTENANCE: u128 = 5_000_000;

// Expiry line up with deribit expirations
#[cfg(feature = "epoch-offset")]
pub const EPOCH_OFFSET: u64 = 115_200; // 32 hours to 8AM Friday UTC from 12am Thursday UTC.
#[cfg(not(feature = "epoch-offset"))]
pub const EPOCH_OFFSET: u64 = 0;

pub const EPSILON_ERROR: u64 = 1_000;

pub const BID_ORDERS_INDEX: usize = 0;
pub const ASK_ORDERS_INDEX: usize = 1;
