use crate::*;
use bytemuck::{Pod, Zeroable};
use std::convert::{From, TryInto};

#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct ProductGreeks {
    pub delta: u64,
    pub vega: AnchorDecimal,
    pub volatility: AnchorDecimal,
} // 40

#[zero_copy]
#[derive(Default)]
#[repr(packed)]
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
#[repr(packed)]
pub struct Greeks {
    pub nonce: u8,                                       // 1
    pub mark_prices: [u64; 46],                          // 8 * 46 = 368
    pub _mark_prices_padding: [u64; 92],                 // 8 * 92 =  736
    pub product_greeks: [ProductGreeks; 22],             // 22 * 40 = 880
    pub _product_greeks_padding: [ProductGreeks; 44],    // 44 * 40 = 1760
    pub update_timestamp: [u64; 2],                      // 16
    pub _update_timestamp_padding: [u64; 4],             // 32
    pub retreat_expiration_timestamp: [u64; 2],          // 16
    pub _retreat_expiration_timestamp_padding: [u64; 4], // 32
    pub interest_rate: [i64; 2],                         // 16
    pub _interest_rate_padding: [i64; 4],                // 32
    pub nodes: [u64; 5],                                 // 40
    pub volatility: [u64; 10],                           // 80
    pub _volatility_padding: [u64; 20],                  // 160
    pub node_keys: [Pubkey; 138],                        // 138 * 32 = 4416
    pub halt_force_pricing: [bool; 6],                   // 6
    pub _padding: [u8; 1641],                            // 1641
} // 10232

impl Greeks {
    pub fn get_mark_prices_slice(&self, expiry_index: usize) -> &[u64] {
        let head = expiry_index * NUM_PRODUCTS_PER_SERIES;
        &self.mark_prices[head..head + NUM_PRODUCTS_PER_SERIES]
    }

    pub fn get_product_greeks_slice(&self, expiry_index: usize) -> &[ProductGreeks] {
        let head = expiry_index * NUM_STRIKES;
        &self.product_greeks[head..head + NUM_STRIKES]
    }

    pub fn get_volatility_slice(&self, expiry_index: usize) -> &[u64] {
        let head = expiry_index * VOLATILITY_POINTS;
        &self.volatility[head..head + VOLATILITY_POINTS]
    }

    pub fn get_futures_price(&self, expiry_index: usize) -> u64 {
        self.mark_prices[expiry_index * NUM_PRODUCTS_PER_SERIES + NUM_PRODUCTS_PER_SERIES - 1]
    }
}

#[account(zero_copy)]
#[repr(packed)]
pub struct ZetaGroup {
    pub nonce: u8,                                // 1
    pub vault_nonce: u8,                          // 1
    pub insurance_vault_nonce: u8,                // 1
    pub front_expiry_index: u8,                   // 1
    pub halt_state: HaltState,                    // 167
    pub underlying_mint: Pubkey,                  // 32
    pub oracle: Pubkey,                           // 32
    pub greeks: Pubkey,                           // 32
    pub pricing_parameters: PricingParameters,    // 112
    pub margin_parameters: MarginParameters,      // 120
    pub products: [Product; 46],                  // 138 * 43 = 5934
    pub products_padding: [Product; 92],          //
    pub expiry_series: [ExpirySeries; 2],         // 32 * 6 = 192
    pub expiry_series_padding: [ExpirySeries; 4], //
    pub total_insurance_vault_deposits: u64,      // 8
    pub asset: Asset,                             // 1
    pub expiry_interval_seconds: u32,             // 4
    pub new_expiry_threshold_seconds: u32,        // 4
    pub padding: [u8; 1054],                      // 1054
} // 7696

#[zero_copy]
#[repr(packed)]
pub struct HaltState {
    _halted: bool,
    _spot_price: u64, // Set with precision 6.
    _timestamp: u64,
    _mark_prices_set: [bool; 2],
    _mark_prices_set_padding: [bool; 4],
    _market_nodes_cleaned: [bool; 2],
    _market_nodes_cleaned_padding: [bool; 4],
    _market_cleaned: [bool; 46],
    _market_cleaned_padding: [bool; 92],
} // 1 + 8 + 8 + 6 + 6 + 46 + 92 = 167

#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct PricingParameters {
    pub option_trade_normalizer: AnchorDecimal, // 16
    pub future_trade_normalizer: AnchorDecimal, // 16
    pub max_volatility_retreat: AnchorDecimal,  // 16
    pub max_interest_retreat: AnchorDecimal,    // 16
    pub max_delta: u64,                         // 8
    pub min_delta: u64,                         // 8
    pub min_volatility: u64,                    // 8
    pub max_volatility: u64,                    // 8
    pub min_interest_rate: i64,                 // 8
    pub max_interest_rate: i64,                 // 8
} // 112

#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct MarginParameters {
    // Futures
    pub future_margin_initial: u64,
    pub future_margin_maintenance: u64,

    // Options initial
    pub option_mark_percentage_long_initial: u64,
    pub option_spot_percentage_long_initial: u64,
    pub option_spot_percentage_short_initial: u64,
    pub option_dynamic_percentage_short_initial: u64,

    // Options maintenance
    pub option_mark_percentage_long_maintenance: u64,
    pub option_spot_percentage_long_maintenance: u64,
    pub option_spot_percentage_short_maintenance: u64,
    pub option_dynamic_percentage_short_maintenance: u64,

    // Other parameters
    pub option_short_put_cap_percentage: u64,
    pub padding: [u8; 32],
} // 120 bytes.

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
            Err(_) => wrap_error!(Err(error!(FuzeErrorCode::InvalidProductMarketKey))),
            Ok(i) => Ok((i, self.get_expiry_index_by_product_index(i))),
        }
    }

    pub fn get_product_index_by_key(&self, market: &Pubkey) -> Result<usize> {
        let index = self
            .products
            .binary_search_by_key(&market, |product| &product.market);

        match index {
            Err(_) => wrap_error!(Err(error!(FuzeErrorCode::InvalidProductMarketKey))),
            Ok(i) => Ok(i),
        }
    }

    pub fn get_expiry_series_by_key(&self, market: &Pubkey) -> Result<&ExpirySeries> {
        let index = self
            .products
            .binary_search_by_key(&market, |product| &product.market);

        match index {
            Err(_) => wrap_error!(Err(error!(FuzeErrorCode::InvalidProductMarketKey))),
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
            return Err(error!(FuzeErrorCode::MarketNotLive));
        }

        let products = self.get_products_slice(expiry_index);
        // We don't need to check product.dirty as status implies that.
        // We only need to check a singular product for strike set in the series.
        if !products[0].strike.is_set() {
            return Err(error!(FuzeErrorCode::ProductStrikeUninitialized));
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
#[repr(packed)]
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
#[repr(packed)]
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
            return Err(error!(FuzeErrorCode::ProductStrikeUninitialized));
        }
        Ok(self.value)
    }
}

#[zero_copy]
#[repr(packed)]
pub struct Product {
    // Serum market
    pub market: Pubkey,
    pub strike: Strike,
    // Tracks whether the market has been wiped after expiration
    pub dirty: bool,
    pub kind: Kind,
} // 32 + 9 + 1 + 1 = 43 bytes

#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct Position {
    pub size: i64,
    pub cost_of_trades: u64,
} // 16

impl Position {
    pub fn check_open(&self, side: Side) -> bool {
        match side {
            Side::Bid => self.size >= 0,
            Side::Ask => self.size <= 0,
            _ => false,
        }
    }

    pub fn empty(&self) -> bool {
        self.size == 0 && self.cost_of_trades == 0
    }

    pub fn size_abs(&self) -> u64 {
        self.size.abs() as u64
    }

    pub fn get_unrealized_pnl(&self, mark_price: u64) -> i64 {
        if self.size == 0 {
            0
        } else if self.size > 0 {
            (self.size as i128)
                .checked_mul(mark_price as i128)
                .unwrap()
                .checked_div(POSITION_PRECISION_DENOMINATOR as i128)
                .unwrap()
                .checked_sub(self.cost_of_trades as i128)
                .unwrap()
                .try_into()
                .unwrap()
        } else {
            (self.size as i128)
                .checked_mul(mark_price as i128)
                .unwrap()
                .checked_div(POSITION_PRECISION_DENOMINATOR as i128)
                .unwrap()
                .checked_add(self.cost_of_trades as i128)
                .unwrap()
                .try_into()
                .unwrap()
        }
    }
}

#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct OrderState {
    pub closing_orders: u64,
    pub opening_orders: [u64; 2],
} // 24

impl OrderState {
    pub fn has_active_orders(&self) -> bool {
        self.opening_orders[BID_ORDERS_INDEX] != 0
            || self.opening_orders[ASK_ORDERS_INDEX] != 0
            || self.closing_orders != 0
    }
}

#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct ProductLedger {
    pub position: Position,
    pub order_state: OrderState,
} // 40

impl ProductLedger {
    pub fn size(&self) -> i64 {
        self.position.size
    }

    pub fn empty(&self) -> bool {
        self.position.empty() && !self.order_state.has_active_orders()
    }

    pub fn get_initial_margin(
        &self,
        mark_price: u64,
        product: &Product,
        spot: u64,
        margin_parameters: &MarginParameters,
    ) -> u64 {
        let strike: u64 = match product.strike.get_strike() {
            Ok(strike) => strike,
            Err(_) => return 0,
        };

        let mut long_lots: u64 = self.order_state.opening_orders[BID_ORDERS_INDEX];
        let mut short_lots: u64 = self.order_state.opening_orders[ASK_ORDERS_INDEX];
        if self.position.size > 0 {
            long_lots = long_lots.checked_add(self.position.size_abs()).unwrap();
        } else if self.position.size < 0 {
            short_lots = short_lots.checked_add(self.position.size_abs()).unwrap();
        }

        let mut long_initial_margin: u128 = 0;
        let mut short_initial_margin: u128 = 0;

        if long_lots > 0 {
            long_initial_margin = (long_lots as u128)
                .checked_mul(
                    get_initial_margin_per_lot(
                        spot,
                        strike,
                        mark_price,
                        product.kind,
                        Side::Bid,
                        margin_parameters,
                    )
                    .unwrap()
                    .try_into()
                    .unwrap(),
                )
                .unwrap();
        }

        if short_lots > 0 {
            short_initial_margin = (short_lots as u128)
                .checked_mul(
                    get_initial_margin_per_lot(
                        spot,
                        strike,
                        mark_price,
                        product.kind,
                        Side::Ask,
                        margin_parameters,
                    )
                    .unwrap()
                    .try_into()
                    .unwrap(),
                )
                .unwrap();
        }

        if product.kind == Kind::Future {
            if long_lots > short_lots {
                return long_initial_margin
                    .checked_div(POSITION_PRECISION_DENOMINATOR)
                    .unwrap()
                    .try_into()
                    .unwrap();
            } else {
                return short_initial_margin
                    .checked_div(POSITION_PRECISION_DENOMINATOR)
                    .unwrap()
                    .try_into()
                    .unwrap();
            }
        }

        long_initial_margin
            .checked_add(short_initial_margin)
            .unwrap()
            .checked_div(POSITION_PRECISION_DENOMINATOR)
            .unwrap()
            .try_into()
            .unwrap()
    }

    pub fn get_maintenance_margin(
        &self,
        mark_price: u64,
        product: &Product,
        spot: u64,
        margin_parameters: &MarginParameters,
    ) -> u64 {
        if self.position.size == 0 {
            return 0;
        }

        let strike: u64 = match product.strike.get_strike() {
            Ok(strike) => strike,
            Err(_) => return 0,
        };

        let maintenance_margin_per_lot = get_maintenance_margin_per_lot(
            spot,
            strike,
            mark_price,
            product.kind,
            self.position.size >= 0,
            margin_parameters,
        )
        .unwrap();

        (self.position.size_abs() as u128)
            .checked_mul(maintenance_margin_per_lot as u128)
            .unwrap()
            .checked_div(POSITION_PRECISION_DENOMINATOR)
            .unwrap() as u64
    }

    pub fn get_maintenance_margin_including_orders(
        &self,
        mark_price: u64,
        product: &Product,
        spot: u64,
        margin_parameters: &MarginParameters,
    ) -> u64 {
        let strike: u64 = match product.strike.get_strike() {
            Ok(strike) => strike,
            Err(_) => return 0,
        };

        let mut long_lots: u128 = self.order_state.opening_orders[BID_ORDERS_INDEX].into();
        let mut short_lots: u128 = self.order_state.opening_orders[ASK_ORDERS_INDEX].into();

        if self.position.size > 0 {
            long_lots = long_lots
                .checked_add(self.position.size_abs() as u128)
                .unwrap();
        } else {
            short_lots = long_lots
                .checked_add(self.position.size_abs() as u128)
                .unwrap();
        }

        let mut maintenance_margin_long = 0;
        let mut maintenance_margin_short = 0;

        if long_lots > 0 {
            maintenance_margin_long = long_lots
                .checked_mul(
                    get_maintenance_margin_per_lot(
                        spot,
                        strike,
                        mark_price,
                        product.kind,
                        true,
                        margin_parameters,
                    )
                    .unwrap() as u128,
                )
                .unwrap()
                .checked_div(POSITION_PRECISION_DENOMINATOR)
                .unwrap() as u64;
        }

        if short_lots > 0 {
            maintenance_margin_short = short_lots
                .checked_mul(
                    get_maintenance_margin_per_lot(
                        spot,
                        strike,
                        mark_price,
                        product.kind,
                        false,
                        margin_parameters,
                    )
                    .unwrap() as u128,
                )
                .unwrap()
                .checked_div(POSITION_PRECISION_DENOMINATOR)
                .unwrap() as u64;
        }

        maintenance_margin_long
            .checked_add(maintenance_margin_short)
            .unwrap()
    }

    pub fn get_margin_market_maker_concession(
        &self,
        mark_price: u64,
        product: &Product,
        spot: u64,
        margin_parameters: &MarginParameters,
        concession_percentage: u8,
    ) -> u64 {
        let strike: u64 = match product.strike.get_strike() {
            Ok(strike) => strike,
            Err(_) => return 0,
        };

        let long_lots: u64 = self.order_state.opening_orders[BID_ORDERS_INDEX];
        let short_lots: u64 = self.order_state.opening_orders[ASK_ORDERS_INDEX];
        let mut long_initial_margin: u128 = 0;
        let mut short_initial_margin: u128 = 0;

        if long_lots > 0 {
            long_initial_margin = (long_lots as u128)
                .checked_mul(
                    get_initial_margin_per_lot(
                        spot,
                        strike,
                        mark_price,
                        product.kind,
                        Side::Bid,
                        margin_parameters,
                    )
                    .unwrap()
                    .try_into()
                    .unwrap(),
                )
                .unwrap();
        }

        if short_lots > 0 {
            short_initial_margin = (short_lots as u128)
                .checked_mul(
                    get_initial_margin_per_lot(
                        spot,
                        strike,
                        mark_price,
                        product.kind,
                        Side::Ask,
                        margin_parameters,
                    )
                    .unwrap()
                    .try_into()
                    .unwrap(),
                )
                .unwrap();
        }

        // Apply the concession
        let total_initial_margin = long_initial_margin
            .checked_add(short_initial_margin)
            .unwrap()
            .checked_mul(concession_percentage.into())
            .unwrap()
            .checked_div(100)
            .unwrap();

        let maintenance_margin_per_lot = get_maintenance_margin_per_lot(
            spot,
            strike,
            mark_price,
            product.kind,
            self.position.size >= 0,
            margin_parameters,
        )
        .unwrap();

        // Normalize it with the initial_margin
        let maintenance_margin = (self.position.size_abs() as u128)
            .checked_mul(maintenance_margin_per_lot as u128)
            .unwrap();

        maintenance_margin
            .checked_add(total_initial_margin)
            .unwrap()
            .checked_div(POSITION_PRECISION_DENOMINATOR)
            .unwrap()
            .try_into()
            .unwrap()
    }
}

#[account(zero_copy)]
#[repr(packed)]
pub struct SpreadAccount {
    pub authority: Pubkey,                 // 32
    pub nonce: u8,                         // 1
    pub balance: u64,                      // 8
    pub series_expiry: [u64; 6],           // 48
    pub positions: [Position; 46],         // 16 * 138 = 2208
    pub positions_padding: [Position; 92], //
    pub asset: Asset,                      // 1
    pub padding: [u8; 262],                // 262
} // 2560

impl SpreadAccount {
    pub fn empty(&self) -> bool {
        if self.has_positions() {
            return false;
        }
        self.balance == 0
    }

    pub fn has_positions(&self) -> bool {
        self.positions.iter().any(|&x| !x.empty())
    }

    pub fn get_positions_slice_mut(&mut self, expiry_index: usize) -> &mut [Position] {
        let head = expiry_index * NUM_PRODUCTS_PER_SERIES;
        &mut self.positions[head..head + NUM_PRODUCTS_PER_SERIES]
    }

    pub fn get_positions_slice(&self, expiry_index: usize) -> &[Position] {
        let head = expiry_index * NUM_PRODUCTS_PER_SERIES;
        &self.positions[head..head + NUM_PRODUCTS_PER_SERIES]
    }

    pub fn has_position_in_expiry_index(&self, index: usize) -> bool {
        self.get_positions_slice(index)
            .iter()
            .find(|x| x.size != 0)
            .is_some()
    }
}

#[account(zero_copy)]
#[repr(packed)]
pub struct MarginAccount {
    pub authority: Pubkey,                             // 32
    pub nonce: u8,                                     // 1
    pub balance: u64,                                  // 8
    pub force_cancel_flag: bool,                       // 1
    pub open_orders_nonce: [u8; 138],                  // 138
    pub series_expiry: [u64; 6],                       // 48
    pub product_ledgers: [ProductLedger; 46],          // 138 * 40 = 5520
    pub _product_ledgers_padding: [ProductLedger; 92], //
    pub rebalance_amount: i64,                         // 8
    pub asset: Asset,                                  // 1
    pub account_type: MarginAccountType,               // 1
    pub _padding: [u8; 386],                           // 386
} // 6144

impl MarginAccount {
    pub fn get_product_ledgers_slice_mut(&mut self, expiry_index: usize) -> &mut [ProductLedger] {
        let head = expiry_index * NUM_PRODUCTS_PER_SERIES;
        &mut self.product_ledgers[head..head + NUM_PRODUCTS_PER_SERIES]
    }

    pub fn get_product_ledgers_slice(&self, expiry_index: usize) -> &[ProductLedger] {
        let head = expiry_index * NUM_PRODUCTS_PER_SERIES;
        &self.product_ledgers[head..head + NUM_PRODUCTS_PER_SERIES]
    }

    // Calculates the total initial margin for all open orders and positions.
    pub fn get_initial_margin(&self, greeks: &Greeks, zeta_group: &ZetaGroup, spot: u64) -> u64 {
        let initial_margin_requirement = self
            .product_ledgers
            .iter()
            .enumerate()
            .map(|(i, ledger)| {
                ledger.get_initial_margin(
                    greeks.mark_prices[i],
                    &zeta_group.products[i],
                    spot,
                    &zeta_group.margin_parameters,
                )
            })
            .sum();

        initial_margin_requirement
    }

    // Calculates the total maintenance margin for all positions only.
    pub fn get_maintenance_margin(
        &self,
        greeks: &Greeks,
        zeta_group: &ZetaGroup,
        spot: u64,
    ) -> u64 {
        let maintenance_margin_requirement = self
            .product_ledgers
            .iter()
            .enumerate()
            .map(|(i, product_ledgers)| {
                product_ledgers.get_maintenance_margin(
                    greeks.mark_prices[i],
                    &zeta_group.products[i],
                    spot,
                    &zeta_group.margin_parameters,
                )
            })
            .sum();

        maintenance_margin_requirement
    }

    pub fn get_unrealized_pnl(&self, greeks: &Greeks) -> i64 {
        self.product_ledgers
            .iter()
            .enumerate()
            .map(|(i, product_ledger)| {
                (product_ledger
                    .position
                    .get_unrealized_pnl(greeks.mark_prices[i]) as i128) as i64
            })
            .sum()
    }

    pub fn get_maintenance_margin_including_orders(
        &self,
        greeks: &Greeks,
        zeta_group: &ZetaGroup,
        spot: u64,
    ) -> u64 {
        let maintenance_margin_requirement = self
            .product_ledgers
            .iter()
            .enumerate()
            .map(|(i, product_ledger)| {
                product_ledger.get_maintenance_margin_including_orders(
                    greeks.mark_prices[i],
                    &zeta_group.products[i],
                    spot,
                    &zeta_group.margin_parameters,
                )
            })
            .sum();

        maintenance_margin_requirement
    }

    pub fn is_market_maker(&self) -> bool {
        self.account_type == MarginAccountType::MarketMaker
    }

    pub fn get_margin_market_maker_concession(
        &self,
        greeks: &Greeks,
        zeta_group: &ZetaGroup,
        spot: u64,
        concession: u8,
    ) -> u64 {
        let maintenance_margin_requirement = self
            .product_ledgers
            .iter()
            .enumerate()
            .map(|(i, product_ledger)| {
                product_ledger.get_margin_market_maker_concession(
                    greeks.mark_prices[i],
                    &zeta_group.products[i],
                    spot,
                    &zeta_group.margin_parameters,
                    concession,
                )
            })
            .sum();

        maintenance_margin_requirement
    }

    pub fn get_margin_requirement(
        &self,
        margin_type: MarginRequirement,
        greeks: &Greeks,
        zeta_group: &ZetaGroup,
        native_spot: u64,
        margin_concession_percentage: Option<u8>,
    ) -> u64 {
        match margin_type {
            MarginRequirement::Initial => {
                self.get_initial_margin(&greeks, &zeta_group, native_spot)
            }
            MarginRequirement::Maintenance => {
                self.get_maintenance_margin(&greeks, &zeta_group, native_spot)
            }
            MarginRequirement::MaintenanceIncludingOrders => {
                self.get_maintenance_margin_including_orders(&greeks, &zeta_group, native_spot)
            }
            MarginRequirement::MarketMakerConcession => {
                assert!(self.is_market_maker());
                assert!(margin_concession_percentage.is_some());
                let margin_concession_percentage = margin_concession_percentage.unwrap();
                assert!(margin_concession_percentage > 0 && margin_concession_percentage <= 100);
                self.get_margin_market_maker_concession(
                    &greeks,
                    &zeta_group,
                    native_spot,
                    margin_concession_percentage,
                )
            }
        }
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
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
    Uninitialized = 0,
    Call = 1,
    Put = 2,
    Future = 3,
    Perp = 4,
}

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Copy)]
pub enum Side {
    Uninitialized = 0,
    Bid = 1,
    Ask = 2,
}

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Copy)]
pub enum OrderType {
    Limit = 0,
    PostOnly = 1,
    FillOrKill = 2,
}

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Copy)]
pub enum Asset {
    SOL = 0,
    BTC = 1,
    ETH = 2,
    UNDEFINED = 255,
}

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Copy)]
pub enum MarginAccountType {
    Normal = 0,
    MarketMaker = 1,
}

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Copy)]
pub enum MovementType {
    Undefined = 0,
    Lock = 1,   // Margin account to spread
    Unlock = 2, // Spread to margin account
}

pub enum MarginRequirement {
    Initial = 0,     // Initial margin for all open orders and positions; used on new inserts
    Maintenance = 1, // Maintenance margin for all positions used on liquidations
    MaintenanceIncludingOrders = 2, // Maintenance margin for all positions and open orders; used on new inserts that only close existing positions
    MarketMakerConcession = 3,      // See usage.
}
