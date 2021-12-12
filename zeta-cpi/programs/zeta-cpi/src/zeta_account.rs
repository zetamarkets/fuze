use crate::*;
use bytemuck::{Pod, Zeroable};
use std::convert::{From, TryFrom};

#[zero_copy]
#[derive(Default)]
pub struct ProductGreeks {
    pub delta: u64,
    pub vega: AnchorDecimal,
    pub volatility: AnchorDecimal,
}

#[zero_copy]
#[derive(Default)]
pub struct AnchorDecimal {
    pub flags: u32,
    pub hi: u32,
    pub lo: u32,
    pub mid: u32,
}

impl From<Decimal> for AnchorDecimal {
    fn from(decimal: Decimal) -> AnchorDecimal {
        AnchorDecimal {
            flags: decimal.flags,
            hi: decimal.hi,
            lo: decimal.lo,
            mid: decimal.mid,
        }
    }
}

impl From<AnchorDecimal> for Decimal {
    fn from(decimal: AnchorDecimal) -> Decimal {
        Decimal {
            flags: decimal.flags,
            hi: decimal.hi,
            lo: decimal.lo,
            mid: decimal.mid,
        }
    }
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for AnchorDecimal {}

#[cfg(target_endian = "little")]
unsafe impl Pod for AnchorDecimal {}

#[account(zero_copy)]
pub struct Greeks {
    pub nonce: u8,
    pub mark_prices: [u64; 46],
    pub _mark_prices_padding: [u64; 92],

    pub product_greeks: [ProductGreeks; 22], // TOTAL_MARKETS
    pub _product_greeks_padding: [ProductGreeks; 44],

    pub update_timestamp: [u64; 2],          // per expiration.
    pub _update_timestamp_padding: [u64; 4], // per expiration.

    pub retreat_expiration_timestamp: [u64; 2], // per expiration.
    pub _retreat_expiration_timestamp_padding: [u64; 4], // per expiration.

    pub interest_rate: [u64; 2],
    pub _interest_rate_padding: [u64; 4],

    pub nodes: [u64; 5],                // 5 per expiration // f/k space nodes
    pub volatility: [u64; 10],          // 5 per expiration // volatility nodes
    pub _volatility_padding: [u64; 20], // 5 per expiration // volatility nodes

    pub node_keys: [Pubkey; 138],
}

impl Greeks {
    pub fn get_futures_price(&self, expiry_index: usize) -> u64 {
        self.mark_prices[expiry_index * NUM_PRODUCTS_PER_SERIES + NUM_PRODUCTS_PER_SERIES - 1]
    }
}

#[zero_copy]
#[derive(Default)]
pub struct MarginParameters {
    pub future_margin_initial: u64,
    pub future_margin_maintenance: u64,
    pub option_mark_percentage_long_initial: u64,
    pub option_spot_percentage_long_initial: u64,
    pub option_spot_percentage_short_initial: u64,
    pub option_base_percentage_short_initial: u64,
    pub option_mark_percentage_long_maintenance: u64,
    pub option_spot_percentage_long_maintenance: u64,
    pub option_spot_percentage_short_maintenance: u64,
    pub option_base_percentage_short_maintenance: u64,
}

#[account(zero_copy)]
pub struct ZetaGroup {
    pub nonce: u8,
    pub front_expiry_index: u8,
    pub underlying_mint: Pubkey,
    pub oracle: Pubkey,
    pub greeks: Pubkey,
    pub pricing_parameters: PricingParameters,
    pub margin_parameters: MarginParameters,
    pub padding: [u8; 122], // 256 - 98 - 36

    pub products: [Product; 46],
    pub products_padding: [Product; 92],
    pub expiry_series: [ExpirySeries; 2],
    pub expiry_series_padding: [ExpirySeries; 4],

    pub vault_nonce: u8,
    pub insurance_vault_nonce: u8,
    pub total_insurance_vault_deposits: u64,
}

impl ZetaGroup {
    pub fn get_strike(&self, index: usize) -> Result<u64> {
        self.products[index].strike.get_strike()
    }

    pub fn get_products_slice(&self, expiry_index: usize) -> &[Product] {
        let head = expiry_index * NUM_PRODUCTS_PER_SERIES;
        &self.products[head..head + NUM_PRODUCTS_PER_SERIES]
    }

    pub fn get_product_and_expiry_index_by_key(&self, market: &Pubkey) -> Result<(usize, usize)> {
        let index = self
            .products
            .binary_search_by_key(&market, |product| &product.market);

        match index {
            Err(_) => wrap_error!(Err(ErrorCode::InvalidProductMarketKey.into())),
            Ok(i) => Ok((i, self.get_expiry_index_by_product_index(i))),
        }
    }

    pub fn get_product_index_by_key(&self, market: &Pubkey) -> Result<usize> {
        let index = self
            .products
            .binary_search_by_key(&market, |product| &product.market);

        match index {
            Err(_) => wrap_error!(Err(ErrorCode::InvalidProductMarketKey.into())),
            Ok(i) => Ok(i),
        }
    }

    pub fn get_expiry_series_by_key(&self, market: &Pubkey) -> Result<&ExpirySeries> {
        let index = self
            .products
            .binary_search_by_key(&market, |product| &product.market);

        match index {
            Err(_) => wrap_error!(Err(ErrorCode::InvalidProductMarketKey.into())),
            Ok(i) => Ok(self.get_expiry_series_by_product_index(i)),
        }
    }

    pub fn get_expiry_series_by_product_index(&self, index: usize) -> &ExpirySeries {
        &self.expiry_series[self.get_expiry_index_by_product_index(index)]
    }

    pub fn get_expiry_index_by_product_index(&self, index: usize) -> usize {
        assert!(index < self.products.len());
        let expiry_index = index.checked_div(NUM_PRODUCTS_PER_SERIES).unwrap();
        assert!(expiry_index < self.expiry_series.len());
        expiry_index
    }

    /// This function should validate an expiry index is:
    /// 1. Live
    /// 2. Strike is set
    /// 3. Pricing update was within the required intervals.
    pub fn validate_series_tradeable(&self, expiry_index: usize) -> Result<()> {
        let series_status = self.expiry_series[expiry_index].status()?;
        if series_status != ExpirySeriesStatus::Live {
            msg!("Series status = {:?}", series_status);
            return wrap_error!(Err(ErrorCode::MarketNotLive.into()));
        }

        let products = self.get_products_slice(expiry_index);
        // We don't need to check product.dirty as status implies that.
        // We only need to check a singular product for strike set in the series.
        if !products[0].strike.is_set() {
            return wrap_error!(Err(ErrorCode::ProductStrikeUninitialized.into()));
        }
        Ok(())
    }

    pub fn get_back_expiry_index(&self) -> usize {
        // This is built in with the invariant that on series expiration,
        // the series index of the expired set is changed to be the next
        // latest timestamp.
        // This condition may not hold if we change the series layout.
        match self.front_expiry_index {
            0 => (self.expiry_series.len() - 1),
            _ => (self.front_expiry_index - 1).into(),
        }
    }

    // Return the expiry timestamp that is furthest in the future.
    pub fn get_back_expiry_ts(&self) -> u64 {
        self.expiry_series[self.get_back_expiry_index()].expiry_ts
    }

    // Does a wrapped -1 to the passed in expiry_index
    pub fn get_previous_expiry_index(&self, expiry_index: usize) -> usize {
        match expiry_index {
            0 => (self.expiry_series.len() - 1),
            _ => (expiry_index - 1).into(),
        }
    }
}

#[zero_copy]
#[derive(Default)]
pub struct PricingParameters {
    pub option_trade_normalizer: AnchorDecimal,
    pub future_trade_normalizer: AnchorDecimal,
    pub max_volatility_retreat: AnchorDecimal,
    pub max_interest_retreat: AnchorDecimal,
    pub max_delta: u64,
    pub min_delta: u64,
}

#[zero_copy]
pub struct ExpirySeries {
    pub active_ts: u64,
    pub expiry_ts: u64,
    pub dirty: bool,
    pub padding: [u8; 15], // 32 - 17
}

impl ExpirySeries {
    pub fn status(&self) -> Result<ExpirySeriesStatus> {
        if self.active_ts == u64::default() || self.expiry_ts == u64::default() {
            return Ok(ExpirySeriesStatus::Uninitialized);
        };
        let clock = Clock::get()?;
        let current_ts = clock.unix_timestamp as u64;
        if self.dirty {
            Ok(ExpirySeriesStatus::ExpiredDirty)
        } else if current_ts < self.active_ts {
            Ok(ExpirySeriesStatus::Initialized)
        } else if current_ts >= self.active_ts && current_ts < self.expiry_ts {
            Ok(ExpirySeriesStatus::Live)
        } else {
            Ok(ExpirySeriesStatus::Expired)
        }
    }
}

// To mimic an Option<T> as anchor doesn't support zero_copy Option<T> deserialization yet.
// Also, this implementation saves 7 bytes of space :)
#[zero_copy]
pub struct Strike {
    pub is_set: bool,
    pub value: u64,
}

impl Strike {
    pub fn is_set(&self) -> bool {
        self.is_set
    }

    pub fn get_strike(&self) -> Result<u64> {
        if !self.is_set() {
            return Err(ErrorCode::ProductStrikeUninitialized.into());
        }
        Ok(self.value)
    }
}

#[zero_copy]
pub struct Product {
    // Serum market
    pub market: Pubkey,
    pub strike: Strike,
    // Tracks whether the market has been wiped after expiration
    pub dirty: bool,
    pub kind: Kind,
}

#[zero_copy]
#[derive(Default)]
pub struct Position {
    pub position: i32,
    pub cost_of_trades: u64,
    pub closing_orders: u32,
    pub opening_orders: [u32; 2],
}

impl Position {
    pub fn get_initial_margin(
        &self,
        mark_price: u64,
        product: &Product,
        spot: u64,
        margin_parameters: &MarginParameters,
    ) -> u64 {
        let initial_margin_requirement = match product.strike.get_strike() {
            Ok(strike) => (self.opening_orders[0] as u64)
                .checked_mul(
                    get_initial_margin_per_lot(
                        spot,
                        strike,
                        mark_price,
                        product.kind,
                        Side::Bid,
                        margin_parameters,
                    )
                    .unwrap(),
                )
                .unwrap()
                .checked_add(
                    (self.opening_orders[1] as u64)
                        .checked_mul(
                            get_initial_margin_per_lot(
                                spot,
                                strike,
                                mark_price,
                                product.kind,
                                Side::Ask,
                                margin_parameters,
                            )
                            .unwrap(),
                        )
                        .unwrap(),
                )
                .unwrap(),
            Err(_) => 0,
        };
        initial_margin_requirement
    }

    pub fn get_maintenance_margin(
        &self,
        mark_price: u64,
        product: &Product,
        spot: u64,
        margin_parameters: &MarginParameters,
    ) -> u64 {
        let maintenance_margin_requirement = match product.strike.get_strike() {
            Ok(strike) => (self.position.abs() as u64)
                .checked_mul(
                    get_maintenance_margin_per_lot(
                        spot,
                        strike,
                        mark_price,
                        product.kind,
                        self.position >= 0,
                        margin_parameters,
                    )
                    .unwrap(),
                )
                .unwrap(),
            Err(_) => 0,
        };
        maintenance_margin_requirement
    }

    pub fn get_unrealized_pnl(&self, mark_price: u64) -> i64 {
        if self.position > 0 {
            (self.position as i64)
                .checked_mul(mark_price as i64)
                .unwrap()
                .checked_sub(self.cost_of_trades as i64)
                .unwrap()
        } else {
            (self.position as i64)
                .checked_mul(mark_price as i64)
                .unwrap()
                .checked_add(self.cost_of_trades as i64)
                .unwrap()
        }
    }
}

#[account(zero_copy)]
pub struct MarginAccount {
    pub authority: Pubkey,
    pub nonce: u8,
    pub balance: u64,
    pub force_cancel_flag: bool,

    pub open_orders_nonce: [u8; 138],
    pub series_expiry: [u64; 6], // Tracks the expiration of this index, set to 0 if clean
    pub positions: [Position; 46],
    pub positions_padding: [Position; 92], // For future when we add more expiries.

    pub rebalance_amount: i64,
}

impl MarginAccount {
    pub fn get_initial_margin(&self, greeks: &Greeks, zeta_group: &ZetaGroup, spot: u64) -> u64 {
        let initial_margin_requirement = self
            .positions
            .iter()
            .enumerate()
            .map(|(i, position)| {
                position.get_initial_margin(
                    greeks.mark_prices[i],
                    &zeta_group.products[i],
                    spot,
                    &zeta_group.margin_parameters,
                )
            })
            .sum();
        initial_margin_requirement
    }

    pub fn get_maintenance_margin(
        &self,
        greeks: &Greeks,
        zeta_group: &ZetaGroup,
        spot: u64,
    ) -> u64 {
        let maintenance_margin_requirement = self
            .positions
            .iter()
            .enumerate()
            .map(|(i, position)| {
                position.get_maintenance_margin(
                    greeks.mark_prices[i],
                    &zeta_group.products[i],
                    spot,
                    &zeta_group.margin_parameters,
                )
            })
            .sum();

        msg!(
            "Total Maintenance requirement = {}",
            maintenance_margin_requirement
        );
        maintenance_margin_requirement
    }

    pub fn get_margin_requirement(
        &self,
        greeks: &Greeks,
        zeta_group: &ZetaGroup,
        spot: u64,
    ) -> u64 {
        self.get_initial_margin(greeks, zeta_group, spot)
            .checked_add(self.get_maintenance_margin(greeks, zeta_group, spot))
            .unwrap()
    }

    pub fn get_unrealized_pnl(&self, greeks: &Greeks) -> i64 {
        self.positions
            .iter()
            .enumerate()
            .map(|(i, position)| position.get_unrealized_pnl(greeks.mark_prices[i]))
            .sum()
    }

    pub fn check_margin_requirement(
        &self,
        greeks: &Greeks,
        zeta_group: &ZetaGroup,
        native_spot: u64,
    ) -> bool {
        let pnl = self.get_unrealized_pnl(&greeks);
        let margin_requirement =
            i64::try_from(self.get_margin_requirement(&greeks, &zeta_group, native_spot)).unwrap();
        let buffer = i64::try_from(self.balance)
            .unwrap()
            .checked_add(pnl)
            .unwrap()
            .checked_sub(margin_requirement)
            .unwrap();
        buffer > 0
    }
}

// Enum Types
#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum ExpirySeriesStatus {
    Uninitialized = 0, // Still in default state
    Initialized = 1,   // Initialized but not active yet
    Live = 2,          // Active and trading.
    Expired = 3,       // Intermediate state after active trading
    ExpiredDirty = 4,  // State when series has expired but markets haven't been cleaned
}

#[repr(u8)]
#[derive(PartialEq, Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum Kind {
    Uninitialized = 0,
    Call = 1,
    Put = 2,
    Future = 3,
}

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Copy)]
pub enum Side {
    Uninitialized = 0,
    Bid = 1,
    Ask = 2,
}
