// Currently this code isn't used and is more here for reference on how
// to conduct an auction fully on-chain

#[access_control(auction_phase(&ctx.accounts.vault))]
pub fn validate_market(ctx: Context<ValidateMarket>, delta: u8) -> ProgramResult {
    assert!(delta >= 0 && delta <= 100);
    // Delta values are stored at high precision for pricing
    let native_delta = (delta as u64)
        .checked_mul(10u64.pow(PRICING_PRECISION))
        .unwrap();

    // 1. Instrument selection: select the closest to 1w expiry and specific delta strike
    let zeta_group =
        deserialize_account_info_zerocopy::<ZetaGroup>(&ctx.accounts.zeta_group).unwrap();
    // Get the expiry closest to 1 week
    let closest_expiry_index = 0;
    let closest_expiry_diff = unsigned_abs_diff(
        UNIX_WEEK,
        zeta_group.expiry_series[closest_expiry_index].expiry_ts,
    )
    .unwrap();
    for (i, t) in zeta_group.expiry_series.iter().enumerate() {
        let expiry_diff = unsigned_abs_diff(UNIX_WEEK, t.expiry_ts).unwrap();
        if expiry_diff < closest_expiry_diff {
            closest_expiry_index = i;
        }
    }
    // Get strike closest to specified delta
    let greeks = deserialize_account_info_zerocopy::<Greeks>(&ctx.accounts.greeks).unwrap();
    let product_greeks = greeks.get_product_greeks_slice(closest_expiry_index);
    let closest_delta_index = 0;
    let closest_delta_diff =
        unsigned_abs_diff(native_delta, product_greeks[closest_delta_index].delta).unwrap();
    for (i, g) in product_greeks.iter().enumerate() {
        let delta_diff = unsigned_abs_diff(native_delta, g.delta).unwrap();
        if delta_diff < closest_delta_diff {
            closest_delta_index = i;
        }
    }

    // Sell puts on Zeta for given market
    let market_index = closest_expiry_index * NUM_PRODUCTS_PER_SERIES + NUM_STRIKES + closest_delta_index;

    Ok(())
}

#[access_control(auction_phase(&ctx.accounts.vault))]
pub fn sell_put(ctx: Context<SellPut>) -> ProgramResult {
    msg!("AUCTION: SELL PUT");

    // Sell puts on Zeta for given market
    let market_key = ctx.accounts.marketAccounts.market.key();
    let market = ctx.accounts.marketAccounts.zeta_group.get_product_index_by_key(market_key);
    let strike = market.strike;
    let size = ctx.accounts.vault_usdc.amount.checked_div(strike).unwrap();
    zeta_client::place_order(
        ctx.accounts.zeta_program.clone(),
        ctx.accounts.place_order_cpi_accounts.clone(),
        price,
        size,
        Side::Ask,
    );

    Ok(())
}