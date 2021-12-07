use crate::*;
use std::cell::{RefMut};
use std::ops::DerefMut;
use std::convert::{TryFrom, TryInto};

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
) -> Result<u64> {
  let initial_margin: u128 = match product {
      // 15% of Spot
      Kind::Future => (spot as u128)
          .checked_mul(1500)
          .unwrap()
          .checked_div(10000)
          .unwrap(),
      Kind::Call | Kind::Put => {
          let otm_amount = get_otm_amount(spot, strike, product)?;
          match side {
              Side::Bid => {
                  // min(100% * mark price, 15% of spot)
                  // Place holder calcs in place for 100% * mark price
                  (spot as u128)
                      .checked_mul(1500)
                      .unwrap()
                      .checked_div(10000)
                      .unwrap()
                      .min(
                          (mark as u128)
                              .checked_mul(
                                  MARK_PRICE_PERCENTAGE.checked_mul(10u128.pow(2)).unwrap(),
                              )
                              .unwrap()
                              .checked_div(10000)
                              .unwrap(),
                      )
              }
              Side::Ask => {
                  // max(25% - OTM Amount/spot, 10%)
                  let percentage_multiplier: u128 = 1000u128.max(
                      2500u128
                          .checked_sub(
                              // Convert OTM/Spot to a percentage
                              (otm_amount as u128)
                                  .checked_mul(10000)
                                  .unwrap()
                                  .checked_div(spot as u128)
                                  .unwrap(),
                          )
                          .unwrap_or(0),
                  );
                  // Scale spot by the percentage multiplier
                  (spot as u128)
                      .checked_mul(percentage_multiplier)
                      .unwrap()
                      .checked_div(10000)
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
) -> Result<u64> {
  let maintenance_margin: u128 = match product {
      // 7.5% of Spot
      Kind::Future => (spot as u128)
          .checked_mul(750)
          .unwrap()
          .checked_div(10000)
          .unwrap(),
      Kind::Call | Kind::Put => {
          let otm_amount = get_otm_amount(spot, strike, product)?;
          if long {
              // min(100% * mark price, 7.5% of spot)
              // Place holder calcs for 100% * mark price
              (spot as u128)
                  .checked_mul(750)
                  .unwrap()
                  .checked_div(10000)
                  .unwrap()
                  .min(
                      (mark as u128)
                          .checked_mul(10000)
                          .unwrap()
                          .checked_div(10000)
                          .unwrap(),
                  )
          } else {
              // max((12.5% - OTM Amount/spot)*spot, 5% * spot)
              let percentage_multiplier: u128 = 500u128.max(
                  1250u128
                      .checked_sub(
                          (otm_amount as u128)
                              .checked_mul(10000)
                              .unwrap()
                              .checked_div(spot as u128)
                              .unwrap(),
                      )
                      .unwrap_or(0),
              );

              (spot as u128)
                  .checked_mul(percentage_multiplier)
                  .unwrap()
                  .checked_div(10000)
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