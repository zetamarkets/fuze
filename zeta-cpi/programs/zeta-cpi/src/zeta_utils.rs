use crate::*;
use std::cell::RefMut;
use std::convert::{TryFrom, TryInto};
use std::ops::DerefMut;

#[macro_export]
macro_rules! wrap_error {
    ($err:expr) => {{
        msg!("Error thrown at {}:{}", file!(), line!());
        $err
    }};
}

pub fn deserialize_account_info_zerocopy<'a, T: bytemuck::Pod>(
    account_info: &'a AccountInfo,
) -> Result<RefMut<'a, T>> {
    let data = account_info.try_borrow_mut_data()?;
    Ok(RefMut::map(data, |data| {
        bytemuck::from_bytes_mut(&mut data.deref_mut()[8..])
    }))
}

#[inline(never)]
pub fn deserialize_account_info<'a, T: AccountSerialize + AccountDeserialize + Owner + Clone>(
    account_info: &AccountInfo<'a>,
) -> Result<T> {
    let mut data: &[u8] = &account_info.try_borrow_data()?;
    Ok(T::try_deserialize_unchecked(&mut data)?)
}

pub fn get_otm_amount(spot: u64, strike: u64, product: Kind) -> Result<u64> {
    match product {
        Kind::Call => Ok((strike as i128)
            .checked_sub(spot as i128)
            .unwrap()
            .max(0)
            .try_into()
            .unwrap()),
        Kind::Put => Ok((spot as i128)
            .checked_sub(strike as i128)
            .unwrap()
            .max(0)
            .try_into()
            .unwrap()),
        _ => return wrap_error!(Err(ErrorCode::UnsupportedKind.into())),
    }
}

/// Initial margin for single product
pub fn get_initial_margin_per_lot(
    spot: u64,
    strike: u64,
    mark: u64,
    product: Kind,
    side: Side,
    margin_parameters: &MarginParameters,
) -> Result<u64> {
    let initial_margin: u128 = match product {
        // 15% of Spot
        Kind::Future => (spot as u128)
            .checked_mul(margin_parameters.future_margin_initial.into())
            .unwrap()
            .checked_div(NATIVE_PRECISION_DENOMINATOR)
            .unwrap(),
        Kind::Call | Kind::Put => {
            match side {
                Side::Bid => {
                    // min(100% * mark price, 15% of spot)
                    // Place holder calcs in place for 100% * mark price
                    (spot as u128)
                        .checked_mul(margin_parameters.option_spot_percentage_long_initial.into())
                        .unwrap()
                        .checked_div(NATIVE_PRECISION_DENOMINATOR)
                        .unwrap()
                        .min(
                            (mark as u128)
                                .checked_mul(
                                    margin_parameters.option_mark_percentage_long_initial.into(),
                                )
                                .unwrap()
                                .checked_div(NATIVE_PRECISION_DENOMINATOR)
                                .unwrap(),
                        )
                }
                Side::Ask => {
                    let otm_amount: u128 = get_otm_amount(spot, strike, product)?.into();
                    // max(25% - OTM Amount/spot, 10%)
                    let otm_pct = otm_amount
                        .checked_mul(NATIVE_PRECISION_DENOMINATOR)
                        .unwrap()
                        .checked_div(spot.into())
                        .unwrap();

                    let dynamic_margin_pct =
                        (margin_parameters.option_base_percentage_short_initial as u128)
                            .checked_sub(otm_pct)
                            .unwrap_or(0);

                    let margin_pct = dynamic_margin_pct.max(
                        margin_parameters
                            .option_spot_percentage_short_initial
                            .into(),
                    );
                    margin_pct
                        .checked_mul(spot.into())
                        .unwrap()
                        .checked_div(NATIVE_PRECISION_DENOMINATOR)
                        .unwrap()
                }
                Side::Uninitialized => unreachable!(),
            }
        }
        _ => return wrap_error!(Err(ErrorCode::UnsupportedKind.into())),
    };
    Ok(u64::try_from(initial_margin).unwrap())
}

/// Maintenance margin for single product
pub fn get_maintenance_margin_per_lot(
    spot: u64,
    strike: u64,
    mark: u64,
    product: Kind,
    long: bool,
    margin_parameters: &MarginParameters,
) -> Result<u64> {
    let maintenance_margin: u128 = match product {
        // 7.5% of Spot
        Kind::Future => (spot as u128)
            .checked_mul(margin_parameters.future_margin_maintenance.into())
            .unwrap()
            .checked_div(NATIVE_PRECISION_DENOMINATOR)
            .unwrap(),
        Kind::Call | Kind::Put => {
            if long {
                // min(100% * mark price, 7.5% of spot)
                // Place holder calcs for 100% * mark price
                (spot as u128)
                    .checked_mul(
                        margin_parameters
                            .option_spot_percentage_long_maintenance
                            .into(),
                    )
                    .unwrap()
                    .checked_div(NATIVE_PRECISION_DENOMINATOR)
                    .unwrap()
                    .min(
                        (mark as u128)
                            .checked_mul(
                                margin_parameters
                                    .option_mark_percentage_long_maintenance
                                    .into(),
                            )
                            .unwrap()
                            .checked_div(NATIVE_PRECISION_DENOMINATOR)
                            .unwrap(),
                    )
            } else {
                let otm_amount: u128 = get_otm_amount(spot, strike, product)?.into();
                // max((12.5% - OTM Amount/spot)*spot, 5% * spot)
                let otm_pct = otm_amount
                    .checked_mul(NATIVE_PRECISION_DENOMINATOR)
                    .unwrap()
                    .checked_div(spot.into())
                    .unwrap();

                let dynamic_margin_pct: u128 =
                    (margin_parameters.option_spot_percentage_short_maintenance as u128)
                        .checked_sub(otm_pct)
                        .unwrap_or(0);

                let margin_pct = dynamic_margin_pct.max(
                    margin_parameters
                        .option_base_percentage_short_maintenance
                        .into(),
                );

                margin_pct
                    .checked_mul(spot.into())
                    .unwrap()
                    .checked_div(NATIVE_PRECISION_DENOMINATOR)
                    .unwrap()
            }
        }
        _ => return wrap_error!(Err(ErrorCode::UnsupportedKind.into())),
    };
    Ok(u64::try_from(maintenance_margin).unwrap())
}

/// Returns the native oracle price (6.dp)
///
/// # Arguments
///
/// * `oracle` - Oracle account.
pub fn get_native_oracle_price(oracle: &AccountInfo) -> u64 {
    let oracle_price = pyth_client::Price::load(&oracle).unwrap();
    (oracle_price.agg.price as u128)
        .checked_mul(10u128.pow(PLATFORM_PRECISION.into()))
        .unwrap()
        .checked_div(10u128.pow((-oracle_price.expo).try_into().unwrap()))
        .unwrap()
        .try_into()
        .unwrap()
}

/// Returns the market index given an expiry index and index into the slice.
///
/// # Arguments
///
/// * `expiry_index` - Expiry series index.
/// * `product_index` - Index into the products slice. [0..NUM_PRODUCTS_PER_SERIES).
pub fn get_products_slice_market_index(expiry_index: usize, product_index: usize) -> usize {
    expiry_index
        .checked_mul(NUM_PRODUCTS_PER_SERIES)
        .unwrap()
        .checked_add(product_index)
        .unwrap()
}

/// Returns the greeks index for a given market index.
///
/// # Arguments
///
/// * `expiry_index` - Expiry series index.
/// * `market_index` - The market index of the product to get greeks for.
///
/// Returns - index into ZetaGroup.product_greeks.
pub fn get_greeks_index(expiry_index: usize, market_index: usize) -> usize {
    let slice_product_index = market_index % NUM_PRODUCTS_PER_SERIES;
    // There is no greeks index for the futures market index which are
    // in multiples of NUM_PRODUCTS_PER_SERIES - 1.
    assert!(slice_product_index != NUM_PRODUCTS_PER_SERIES - 1);
    // The greeks for calls and puts are identical and thus use the same slice position.
    let slice_position = slice_product_index % NUM_STRIKES;
    expiry_index
        .checked_mul(NUM_STRIKES)
        .unwrap()
        .checked_add(slice_position)
        .unwrap()
}
