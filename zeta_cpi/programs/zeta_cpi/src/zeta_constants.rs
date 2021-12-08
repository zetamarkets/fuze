pub const STATE_SEED: &str = "state";
pub const GREEKS_SEED: &str = "greeks";
pub const MARKET_NODE_SEED: &str = "market-node";
pub const OPEN_ORDERS_SEED: &str = "open-orders";
pub const VAULT_SEED: &str = "vault";
pub const SERUM_VAULT_SEED: &str = "serum-vault";
pub const ZETA_VAULT_SEED: &str = "zeta-vault";
pub const ZETA_GROUP_SEED: &str = "zeta-group";
pub const ZETA_INSURANCE_VAULT_SEED: &str = "zeta-insurance-vault";
pub const WHITELIST_SEED: &str = "whitelist";
pub const USER_INSURANCE_DEPOSIT_SEED: &str = "user-insurance-deposit";
pub const SETTLEMENT_SEED: &str = "settlement";
pub const MARGIN_SEED: &str = "margin";
pub const UNDERLYING_SEED: &str = "underlying";
pub const SERUM_SEED: &str = "serum";
pub const MINT_AUTH_SEED: &str = "mint-auth";
pub const BASE_MINT_SEED: &str = "base-mint";
pub const QUOTE_MINT_SEED: &str = "quote-mint";
pub const MARKET_SEED: &str = "market";
pub const MARKET_INDEXES_SEED: &str = "market-indexes";

pub const PLATFORM_PRECISION: u32 = 6;
pub const PRICING_PRECISION: u32 = 12;

pub const EVENT_CRANK_LIMIT: u16 = 25;
pub const DEFAULT_MINT_LOT_SIZE: u64 = 1;
pub const DISCRIMINATOR_SIZE: usize = 8;
pub const MARK_PRICE_PERCENTAGE: u128 = 100;
pub const LIQUIDATION_REWARD_PERCENTAGE: u64 = 35;
pub const INSURANCE_VAULT_LIQUIDATION_PERCENTAGE: u64 = 10;
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

pub const SECONDS_IN_A_YEAR: u64 = 31_536_000;

pub const VOLATILITY_POINTS: usize = 5;
