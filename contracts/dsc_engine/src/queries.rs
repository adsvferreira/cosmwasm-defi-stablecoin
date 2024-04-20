use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Decimal, Deps, Env, QuerierWrapper, QueryRequest, StdResult,
    Uint128, WasmQuery,
};
#[cfg(not(feature = "library"))]
use cw_asset::AssetInfo;

use crate::msg::{AccountInfoResponse, ConfigResponse, QueryMsg};
use crate::state::{COLLATERAL_DEPOSITED, CONFIG, DSC_MINTED};
use oracle::msg::{FetchPriceResponse, QueryMsg as OracleQueryMsg};
use pyth_sdk_cw::PriceIdentifier;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(&deps)?),
        QueryMsg::CollateralBalanceOfUser {
            user,
            collateral_asset,
        } => to_json_binary(&query_collateral_balance_of_user(
            &deps,
            user,
            collateral_asset,
        )?),
        QueryMsg::UserHealthFactor { user } => {
            to_json_binary(&query_user_health_factor(&deps, user)?)
        }
        QueryMsg::AccountInformation { user } => {
            to_json_binary(&query_account_information(&deps, user)?)
        }
        QueryMsg::AccountCollateralValueUsd { user } => {
            to_json_binary(&query_account_collateral_value_usd(&deps, user)?)
        }
        QueryMsg::CalculateHealthFactor {
            total_dsc_minted,
            collateral_value_usd,
        } => to_json_binary(&calculate_health_factor(
            &deps,
            total_dsc_minted,
            collateral_value_usd,
        )?),
        QueryMsg::GetUsdValue { token, amount } => to_json_binary(&get_usd_value(
            &deps,
            &AssetInfo::Cw20(Addr::unchecked(token)),
            amount,
        )?),
        QueryMsg::GetTokenAmountFromUsd { token, usd_amount } => {
            to_json_binary(&get_token_amount_from_usd(
                &deps,
                AssetInfo::Cw20(Addr::unchecked(token)).to_string(),
                usd_amount,
            )?)
        }
        QueryMsg::GetCollateralTokenPriceFeed { collateral_asset } => {
            to_json_binary(&get_collateral_token_price_feed(&deps, collateral_asset)?)
        }
        QueryMsg::GetCollateralBalanceOfUser { user, token } => {
            to_json_binary(&get_collateral_balance_of_user(&deps, user, token)?)
        }
    }
}

pub fn query_config(deps: &Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.may_load(deps.storage)?.unwrap();
    let config_res = ConfigResponse {
        owner: config.owner,
        assets: config.assets,
        assets_to_feeds: config.assets_to_feeds,
        oracle_address: config.oracle_address,
        pyth_oracle_address: config.pyth_oracle_address,
        dsc_address: config.dsc_address,
        liquidation_threshold: config.liquidation_threshold,
        liquidation_bonus: config.liquidation_bonus,
        min_health_factor: config.min_health_factor,
    };
    Ok(config_res)
}

pub fn query_collateral_balance_of_user(
    deps: &Deps,
    user: String,
    collateral_asset: String,
) -> StdResult<Uint128> {
    let user_addr = deps.api.addr_validate(&user)?;
    let collateral_balance_of_user =
        COLLATERAL_DEPOSITED.may_load(deps.storage, (&user_addr, collateral_asset))?;
    match collateral_balance_of_user {
        Some(balance) => Ok(balance),
        None => Ok(Uint128::zero()),
    }
}

pub fn query_account_collateral_value_usd(deps: &Deps, user_addr: String) -> StdResult<Decimal> {
    get_account_collateral_value(deps, user_addr)
}

pub fn query_account_information(deps: &Deps, user_addr: String) -> StdResult<AccountInfoResponse> {
    get_account_information(deps, user_addr)
}

pub fn query_user_health_factor(deps: &Deps, user: String) -> StdResult<Decimal> {
    Ok(get_health_factor(deps, user)?)
}

pub fn calculate_health_factor(
    deps: &Deps,
    total_dsc_minted: Uint128,
    collateral_value_in_usd: Decimal,
) -> StdResult<Decimal> {
    if total_dsc_minted == Uint128::new(0) {
        Ok(Decimal::new(Uint128::MAX))
    } else {
        let config = CONFIG.load(deps.storage)?;
        let liquidation_threshold = Decimal::percent(config.liquidation_threshold.u128() as u64);
        let collateral_adjusted_for_threshold =
            collateral_value_in_usd.checked_mul(liquidation_threshold)?;
        Ok(collateral_adjusted_for_threshold / Decimal::from_atomics(total_dsc_minted, 6).unwrap())
    }
}

pub fn get_usd_value(deps: &Deps, asset: &AssetInfo, amount: Uint128) -> StdResult<Decimal> {
    let amount_decimals: u32 = 6;
    let config = CONFIG.load(deps.storage)?;
    let asset_denom = asset.to_string();
    let price_feed_id = config.assets_to_feeds.get(&asset_denom).unwrap();
    let oracle_res = query_price_from_oracle(
        &deps.querier,
        config.oracle_address.to_string(),
        config.pyth_oracle_address.to_string(),
        PriceIdentifier::from_hex(price_feed_id).unwrap(),
    )?;
    let asset_price_usd = Decimal::from_atomics(
        Uint128::from(oracle_res.current_price.price as u64),
        oracle_res.current_price.expo.abs() as u32,
    )
    .unwrap();
    let amount = Decimal::from_atomics(amount, amount_decimals).unwrap();
    Ok(amount.checked_mul(asset_price_usd)?)
}

pub fn get_token_amount_from_usd(
    deps: &Deps,
    asset_denom: String,
    usd_amount: Decimal,
) -> StdResult<Decimal> {
    let config = CONFIG.load(deps.storage)?;
    let price_feed_id = config.assets_to_feeds.get(&asset_denom).unwrap();
    let oracle_res = query_price_from_oracle(
        &deps.querier,
        config.oracle_address.to_string(),
        config.pyth_oracle_address.to_string(),
        PriceIdentifier::from_hex(price_feed_id).unwrap(),
    )?;
    let asset_price_usd = Decimal::from_atomics(
        Uint128::from(oracle_res.current_price.price as u64),
        oracle_res.current_price.expo.abs() as u32,
    )
    .unwrap();
    Ok(usd_amount / asset_price_usd)
}

pub fn get_collateral_token_price_feed(deps: &Deps, asset_denom: String) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    return Ok(config
        .assets_to_feeds
        .get(&asset_denom)
        .unwrap()
        .to_string());
}

pub fn get_collateral_balance_of_user(
    deps: &Deps,
    user_addr: String,
    token: String,
) -> StdResult<Uint128> {
    let user_collateral_balance = COLLATERAL_DEPOSITED
        .may_load(deps.storage, (&deps.api.addr_validate(&user_addr)?, token))?;
    match user_collateral_balance {
        Some(balance) => Ok(balance),
        None => Ok(Uint128::zero()),
    }
}

fn get_account_collateral_value(deps: &Deps, user_addr: String) -> StdResult<Decimal> {
    let config = CONFIG.load(deps.storage)?;

    let collateral_list = config.assets;

    let mut user_deposited_balance_usd = Decimal::new(Uint128::zero());

    for collateral_asset in collateral_list {
        let user_collateral_balance = COLLATERAL_DEPOSITED.may_load(
            deps.storage,
            (
                &deps.api.addr_validate(&user_addr)?,
                collateral_asset.inner(),
            ),
        )?;
        let user_collateral_balance_usd: Decimal = match user_collateral_balance {
            Some(balance) => get_usd_value(&deps, &collateral_asset, balance)?,
            None => Decimal::new(Uint128::zero()),
        };
        user_deposited_balance_usd += user_collateral_balance_usd;
    }

    Ok(user_deposited_balance_usd)
}

fn get_account_information(deps: &Deps, user_addr: String) -> StdResult<AccountInfoResponse> {
    let total_dsc_minted =
        DSC_MINTED.may_load(deps.storage, &deps.api.addr_validate(&user_addr)?)?;
    let total_dsc_minted_parsed = match total_dsc_minted {
        Some(amount) => amount,
        None => Uint128::new(0),
    };
    let acc_info = AccountInfoResponse {
        deposited_collateral_in_usd: get_account_collateral_value(&deps, user_addr.clone())?,
        total_dsc_minted: total_dsc_minted_parsed,
    };
    Ok(acc_info)
}

fn get_health_factor(deps: &Deps, user: String) -> StdResult<Decimal> {
    let AccountInfoResponse {
        deposited_collateral_in_usd,
        total_dsc_minted,
    } = get_account_information(deps, user)?;
    Ok(calculate_health_factor(
        deps,
        total_dsc_minted,
        deposited_collateral_in_usd,
    )?)
}

fn query_price_from_oracle(
    querier: &QuerierWrapper,
    oracle_address: String,
    pyth_oracle_address: String,
    price_feed_id: PriceIdentifier,
) -> StdResult<FetchPriceResponse> {
    let asset_price_usd = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: oracle_address,
        msg: to_json_binary(&OracleQueryMsg::FetchPrice {
            pyth_contract_addr: pyth_oracle_address,
            price_feed_id: price_feed_id,
        })?,
    }))?;
    Ok(asset_price_usd)
}
