use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env,
    MessageInfo, QuerierWrapper, QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};
#[cfg(not(feature = "library"))]
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use cw_asset::AssetInfo;

use crate::error::ContractError;
use crate::msg::{AccountInfoResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, COLLATERAL_DEPOSITED, CONFIG, DSC_MINTED};
use oracle::msg::{FetchPriceResponse, QueryMsg as OracleQueryMsg};
use pyth_sdk_cw::PriceIdentifier;

// version info for migration info
const CONTRACT_NAME: &str = "dsc-engine";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.assets.len() != msg.price_feed_ids.len() {
        return Err(ContractError::TokenAddressesAndPriceFeedIdsLengthsDontMatch {});
    }
    let assets_to_feeds = msg
        .assets
        .clone()
        .into_iter()
        .map(|asset| asset.to_string())
        .zip(msg.price_feed_ids.into_iter())
        .collect();

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        assets: msg.assets,
        assets_to_feeds: assets_to_feeds,
        oracle_address: deps.api.addr_validate(&msg.oracle_address)?,
        pyth_oracle_address: deps.api.addr_validate(&msg.pyth_oracle_address)?,
        dsc_address: deps.api.addr_validate(&msg.dsc_address)?,
        liquidation_threshold: msg.liquidation_threshold,
        liquidation_bonus: msg.liquidation_bonus,
        min_health_factor: msg.min_health_factor,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DepositCollateralAndMintDsc {
            collateral_asset,
            amount_collateral,
            amount_dsc_to_mint,
        } => exec::deposit_collateral_and_mint_dsc(
            deps,
            env,
            info,
            collateral_asset,
            amount_collateral,
            amount_dsc_to_mint,
        ),
        ExecuteMsg::RedeemCollateralForDsc {
            collateral_asset,
            amount_collateral,
            amount_dsc_to_burn,
        } => exec::redeem_collateral_for_dsc(
            deps,
            env,
            info,
            collateral_asset,
            amount_collateral,
            amount_dsc_to_burn,
        ),
        ExecuteMsg::BurnDsc { amount_dsc_to_burn } => {
            exec::burn_dsc(deps, env, info, amount_dsc_to_burn)
        }
        ExecuteMsg::RedeemCollateral {
            collateral_asset,
            amount_collateral,
        } => exec::redeem_collateral(deps, info, collateral_asset, amount_collateral),
        ExecuteMsg::Liquidate {
            collateral_asset,
            user,
            debt_to_cover,
        } => exec::liquidate(deps, env, info, collateral_asset, user, debt_to_cover),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::query_config(&deps)?),
        QueryMsg::CollateralBalanceOfUser {
            user,
            collateral_asset,
        } => to_binary(&query::query_collateral_balance_of_user(
            &deps,
            user,
            collateral_asset,
        )?),
        QueryMsg::UserHealthFactor { user } => {
            to_binary(&query::query_user_health_factor(&deps, user)?)
        }
        QueryMsg::AccountInformation { user } => {
            to_binary(&query::query_account_information(&deps, user)?)
        }
        QueryMsg::AccountCollateralValueUsd { user } => {
            to_binary(&query::query_account_collateral_value_usd(&deps, user)?)
        }
        QueryMsg::CalculateHealthFactor {
            total_dsc_minted,
            collateral_value_usd,
        } => to_binary(&query::calculate_health_factor(
            &deps,
            total_dsc_minted,
            collateral_value_usd,
        )?),
        QueryMsg::GetUsdValue { token, amount } => to_binary(&query::get_usd_value(
            &deps,
            &AssetInfo::Cw20(Addr::unchecked(token)),
            amount,
        )?),
        QueryMsg::GetTokenAmountFromUsd { token, usd_amount } => {
            to_binary(&query::get_token_amount_from_usd(
                &deps,
                AssetInfo::Cw20(Addr::unchecked(token)).to_string(),
                usd_amount,
            )?)
        }
        QueryMsg::GetCollateralTokenPriceFeed { collateral_asset } => to_binary(
            &query::get_collateral_token_price_feed(&deps, collateral_asset)?,
        ),
        QueryMsg::GetCollateralBalanceOfUser { user, token } => {
            to_binary(&query::get_collateral_balance_of_user(&deps, user, token)?)
        }
    }
}

mod exec {

    use cosmwasm_std::Storage;

    use super::*;

    pub fn deposit_collateral_and_mint_dsc(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        collateral_asset: AssetInfo,
        amount_collateral: Uint128,
        amount_dsc_to_mint: Uint128,
    ) -> Result<Response, ContractError> {
        /// CHECK IF COLLATERAL ASSET IS VALID
        let config = CONFIG.load(deps.storage)?;
        if !config
            .assets_to_feeds
            .contains_key(&collateral_asset.to_string())
        {
            return Err(ContractError::InvalidCollateralAsset {
                denom: collateral_asset.inner(),
            });
        }

        /// TRANSFER COLLATERAL FROM USER TO CONTRACT
        // If the asset is a token contract, then we need to execute a TransferFrom msg to receive assets
        // If the asset is native token, the pool balance is already increased
        let mut messages: std::vec::Vec<CosmosMsg<Empty>> = vec![];
        if let AssetInfo::Cw20(contract_addr) = &collateral_asset {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: amount_collateral,
                })?,
                funds: vec![],
            }));
        } else {
            if info.funds[0].denom != collateral_asset.inner()
                || info.funds[0].amount != amount_collateral
            {
                return Err(ContractError::MissingNativeFunds {
                    denom: collateral_asset.inner(),
                });
            }
        };

        COLLATERAL_DEPOSITED.update(
            deps.storage,
            (&info.sender, collateral_asset.inner()),
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default() + amount_collateral)
            },
        )?;

        /// MINT DSC TO USER
        /// NOTE: DSC Engine must be declared as minter on DSC CW20 intantiation
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.dsc_address.into_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: info.sender.to_string(),
                amount: amount_dsc_to_mint,
            })?,
            funds: vec![],
        }));

        DSC_MINTED.update(
            deps.storage,
            &info.sender,
            env.block.height,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default() + amount_dsc_to_mint)
            },
        )?;

        /// VERIFY NEW USER HEALTH FACTOR
        revert_if_health_factor_is_broken(&deps, &info.sender);

        let res = Response::new()
            .add_messages(messages)
            .add_attribute("action", "deposit_collateral")
            .add_attribute("from", info.sender)
            .add_attribute("asset", collateral_asset.inner())
            .add_attribute("amount", amount_collateral);

        Ok(res)
    }

    pub fn redeem_collateral_for_dsc(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        collateral_asset: AssetInfo,
        amount_collateral: Uint128,
        amount_dsc_to_burn: Uint128,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if !config
            .assets_to_feeds
            .contains_key(&collateral_asset.to_string())
        {
            return Err(ContractError::InvalidCollateralAsset {
                denom: collateral_asset.inner(),
            });
        }

        let mut messages: std::vec::Vec<CosmosMsg<Empty>> = vec![];

        /// BURN DSC
        /// NOTE: DSC Engine must be declared as minter on DSC CW20 intantiation
        let burn_dsc_msg = _burn_dsc(
            deps.storage,
            &env,
            amount_dsc_to_burn,
            &info.sender,
            &info.sender,
        )?;
        messages.push(burn_dsc_msg);

        /// REDEEM COLLATERAL
        let redeem_collateral_msg = _redeem_collateral(
            deps.storage,
            &collateral_asset,
            amount_collateral,
            &info.sender,
            &info.sender,
        )?;
        messages.push(redeem_collateral_msg);

        /// VERIFY NEW USER HEALTH FACTOR
        revert_if_health_factor_is_broken(&deps, &info.sender);

        let res = Response::new()
            .add_messages(messages)
            .add_attribute("action", "redeem_collateral")
            .add_attribute("from", info.sender)
            .add_attribute("asset", collateral_asset.inner())
            .add_attribute("amount", amount_collateral);

        Ok(res)
    }

    pub fn liquidate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        collateral_asset: AssetInfo,
        user: String,
        debt_to_cover: Decimal, // usd value
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let collateral_token_decimals_precision = 6; // TODO: dynamic
        let decimal_liquidation_bonus_precision =
            Decimal::from_atomics(config.liquidation_bonus, 2)?;
        let starting_user_health_factor = get_health_factor(&deps, user.to_string())?;
        if starting_user_health_factor >= config.min_health_factor {
            return Err(ContractError::HealthFactorOk {});
        }
        let token_amount_from_debt_covered =
            _get_token_amount_from_usd(&deps, collateral_asset.to_string(), debt_to_cover)?;
        let bonus_collateral = token_amount_from_debt_covered * decimal_liquidation_bonus_precision;
        let collateral_to_redeem = token_amount_from_debt_covered + bonus_collateral;
        let precision_adjusted_collateral_to_redeem = collateral_to_redeem
            .checked_mul(Decimal::from_atomics(
                10_u32.pow(collateral_token_decimals_precision as u32),
                0,
            )?)?
            .floor()
            .to_string()
            .parse::<Uint128>()?;

        let user_addr = &deps.api.addr_validate(&user)?;
        let mut messages: std::vec::Vec<CosmosMsg<Empty>> = vec![];

        /// REDEEM COLLATERAL
        let redeem_collateral_msg = _redeem_collateral(
            deps.storage,
            &collateral_asset,
            precision_adjusted_collateral_to_redeem,
            user_addr,
            &info.sender,
        )?;
        messages.push(redeem_collateral_msg);

        /// BURN DSC
        let dsc_token_decimals = 6;
        let precision_adjusted_debt_to_cover = debt_to_cover
            .checked_mul(Decimal::from_atomics(
                10_u32.pow(dsc_token_decimals as u32),
                0,
            )?)?
            .floor()
            .to_string()
            .parse::<Uint128>()?;
        let burn_dsc_msg = _burn_dsc(
            deps.storage,
            &env,
            precision_adjusted_debt_to_cover,
            user_addr,
            &info.sender,
        )?;
        messages.push(burn_dsc_msg);

        let ending_user_health_factor = get_health_factor(&deps, user.clone())?;

        if ending_user_health_factor <= starting_user_health_factor {
            return Err(ContractError::HealthFactorNotImproved {});
        }

        revert_if_health_factor_is_broken(&deps, &info.sender);

        let res = Response::new()
            .add_messages(messages)
            .add_attribute("action", "redeem_collateral")
            .add_attribute("from", &user)
            .add_attribute("asset", collateral_asset.inner())
            .add_attribute("amount", collateral_to_redeem.to_string())
            .add_attribute("action", "liquidate")
            .add_attribute("from", &info.sender)
            .add_attribute(
                "collateral_to_redeem",
                precision_adjusted_collateral_to_redeem,
            )
            .add_attribute(
                "precision_adjusted_debt_to_cover",
                precision_adjusted_debt_to_cover,
            )
            .add_attribute(
                "initial_health_factor",
                starting_user_health_factor.to_string(),
            )
            .add_attribute("final_health_factor", ending_user_health_factor.to_string())
            .add_attribute(
                "token_amount_from_debt_covered",
                token_amount_from_debt_covered.to_string(),
            )
            .add_attribute("bonus_collateral", bonus_collateral.to_string());

        Ok(res)
    }

    pub fn redeem_collateral(
        deps: DepsMut,
        info: MessageInfo,
        collateral_asset: AssetInfo,
        amount_collateral: Uint128,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if !config
            .assets_to_feeds
            .contains_key(&collateral_asset.to_string())
        {
            return Err(ContractError::InvalidCollateralAsset {
                denom: collateral_asset.inner(),
            });
        }

        let mut messages: std::vec::Vec<CosmosMsg<Empty>> = vec![];

        let redeem_collateral_msg = _redeem_collateral(
            deps.storage,
            &collateral_asset,
            amount_collateral,
            &info.sender,
            &info.sender,
        )?;
        messages.push(redeem_collateral_msg);

        revert_if_health_factor_is_broken(&deps, &info.sender);

        let res = Response::new()
            .add_messages(messages)
            .add_attribute("action", "redeem_collateral")
            .add_attribute("from", info.sender)
            .add_attribute("asset", collateral_asset.inner())
            .add_attribute("amount", amount_collateral);
        Ok(res)
    }

    pub fn burn_dsc(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount_dsc_to_burn: Uint128,
    ) -> Result<Response, ContractError> {
        let mut messages: std::vec::Vec<CosmosMsg<Empty>> = vec![];
        let mut burn_dsc_msg = _burn_dsc(
            deps.storage,
            &env,
            amount_dsc_to_burn,
            &info.sender,
            &info.sender,
        )?;
        messages.push(burn_dsc_msg);
        revert_if_health_factor_is_broken(&deps, &info.sender);
        let res = Response::new().add_messages(messages);
        Ok(res)
    }

    fn _redeem_collateral(
        storage: &mut dyn Storage,
        collateral_asset: &AssetInfo,
        amount_collateral: Uint128,
        from: &Addr,
        to: &Addr,
    ) -> Result<CosmosMsg, ContractError> {
        let message: CosmosMsg;
        if let AssetInfo::Cw20(contract_addr) = &collateral_asset {
            message = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: to.to_string(),
                    amount: amount_collateral,
                })?,
                funds: vec![],
            });
        } else {
            message = CosmosMsg::Bank(BankMsg::Send {
                to_address: to.to_string(),
                amount: vec![Coin {
                    denom: collateral_asset.inner(),
                    amount: amount_collateral,
                }],
            });
        };

        COLLATERAL_DEPOSITED.update(
            storage,
            (&from, collateral_asset.inner()),
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default() - amount_collateral)
            },
        )?; // will fail if user hasn't enough deposited collateral deposited

        Ok(message)
    }

    fn _burn_dsc(
        storage: &mut dyn Storage,
        env: &Env,
        amount_dsc_to_burn: Uint128,
        on_behalf_of: &Addr,
        dsc_from: &Addr,
    ) -> Result<CosmosMsg, ContractError> {
        let config = CONFIG.load(storage)?;
        let message = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.dsc_address.into_string(),
            msg: to_binary(&Cw20ExecuteMsg::BurnFrom {
                owner: dsc_from.to_string(),
                amount: amount_dsc_to_burn,
            })?,
            funds: vec![],
        });
        DSC_MINTED.update(
            storage,
            on_behalf_of,
            env.block.height,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default() - amount_dsc_to_burn)
            },
        )?;
        Ok(message)
    }

    fn revert_if_health_factor_is_broken(
        deps: &DepsMut,
        user_addr: &Addr,
    ) -> Result<(), ContractError> {
        let user_health_factor = get_health_factor(&deps, user_addr.to_string())?;
        let config = CONFIG.load(deps.storage)?;

        if user_health_factor < config.min_health_factor {
            return Err(ContractError::BreaksHealthFactor {
                health_factor_value: user_health_factor,
                min_value: config.min_health_factor,
            });
        }
        Ok(())
    }

    fn get_health_factor(deps: &DepsMut, user: String) -> Result<Decimal, ContractError> {
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

    fn calculate_health_factor(
        deps: &DepsMut,
        total_dsc_minted: Uint128,
        collateral_value_in_usd: Decimal,
    ) -> Result<Decimal, ContractError> {
        if total_dsc_minted == Uint128::new(0) {
            Ok(Decimal::new(Uint128::MAX))
        } else {
            let config = CONFIG.load(deps.storage)?;
            let liquidation_threshold =
                Decimal::percent(config.liquidation_threshold.u128() as u64);
            let collateral_adjusted_for_threshold =
                collateral_value_in_usd.checked_mul(liquidation_threshold)?;
            Ok(collateral_adjusted_for_threshold / Decimal::from_atomics(total_dsc_minted, 6)?)
        }
    }

    fn get_account_information(
        deps: &DepsMut,
        user_addr: String,
    ) -> Result<AccountInfoResponse, ContractError> {
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

    fn get_account_collateral_value(
        deps: &DepsMut,
        user_addr: String,
    ) -> Result<Decimal, ContractError> {
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
                Some(balance) => exec::get_usd_value(&deps, &collateral_asset, balance)?,
                None => Decimal::new(Uint128::zero()),
            };
            user_deposited_balance_usd += user_collateral_balance_usd;
        }

        Ok(user_deposited_balance_usd)
    }

    fn get_usd_value(
        deps: &DepsMut,
        asset: &AssetInfo,
        amount: Uint128,
    ) -> Result<Decimal, ContractError> {
        let amount_decimals: u32 = 6;
        let config = CONFIG.load(deps.storage)?;
        let asset_denom = asset.to_string();
        let price_feed_id = config.assets_to_feeds.get(&asset_denom).unwrap();
        let oracle_res = query_price_from_oracle(
            &deps.querier,
            config.oracle_address.to_string(),
            config.pyth_oracle_address.to_string(),
            PriceIdentifier::from_hex(price_feed_id)?,
        )?;
        let asset_price_usd = Decimal::from_atomics(
            Uint128::from(oracle_res.current_price.price as u64),
            oracle_res.current_price.expo.abs() as u32,
        )?;
        let amount = Decimal::from_atomics(amount, amount_decimals)?;
        Ok(amount.checked_mul(asset_price_usd)?)
    }

    fn query_price_from_oracle(
        querier: &QuerierWrapper,
        oracle_address: String,
        pyth_oracle_address: String,
        price_feed_id: PriceIdentifier,
    ) -> Result<FetchPriceResponse, ContractError> {
        let asset_price_usd = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: oracle_address,
            msg: to_binary(&OracleQueryMsg::FetchPrice {
                pyth_contract_addr: pyth_oracle_address,
                price_feed_id: price_feed_id,
            })?,
        }))?;
        Ok(asset_price_usd)
    }

    fn _get_token_amount_from_usd(
        deps: &DepsMut,
        asset_denom: String,
        usd_amount: Decimal,
    ) -> Result<Decimal, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let price_feed_id = config.assets_to_feeds.get(&asset_denom).unwrap();
        let oracle_res = query_price_from_oracle(
            &deps.querier,
            config.oracle_address.to_string(),
            config.pyth_oracle_address.to_string(),
            PriceIdentifier::from_hex(price_feed_id)?,
        )?;
        let asset_price_usd = Decimal::from_atomics(
            Uint128::from(oracle_res.current_price.price as u64),
            oracle_res.current_price.expo.abs() as u32,
        )?;
        Ok(usd_amount / asset_price_usd)
    }
}

mod query {

    use super::*;

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

    pub fn query_account_collateral_value_usd(
        deps: &Deps,
        user_addr: String,
    ) -> StdResult<Decimal> {
        get_account_collateral_value(deps, user_addr)
    }

    pub fn query_account_information(
        deps: &Deps,
        user_addr: String,
    ) -> StdResult<AccountInfoResponse> {
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
            let liquidation_threshold =
                Decimal::percent(config.liquidation_threshold.u128() as u64);
            let collateral_adjusted_for_threshold =
                collateral_value_in_usd.checked_mul(liquidation_threshold)?;
            Ok(collateral_adjusted_for_threshold
                / Decimal::from_atomics(total_dsc_minted, 6).unwrap())
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
            msg: to_binary(&OracleQueryMsg::FetchPrice {
                pyth_contract_addr: pyth_oracle_address,
                price_feed_id: price_feed_id,
            })?,
        }))?;
        Ok(asset_price_usd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coins, Empty};
    use cw20::{BalanceResponse, Cw20Coin, MinterResponse, TokenInfoResponse};
    use cw20_base::contract::{
        execute as cw20_execute, instantiate as cw20_instantiate, query as cw20_query,
    };
    use cw20_base::msg::{InstantiateMsg as Cw20InstantiateMsg, QueryMsg as Cw20QueryMsg};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use dsc::contract::{
        execute as dsc_execute, instantiate as dsc_instantiate, query as dsc_query,
    };
    use mock_pyth::contract::{
        execute as mock_pyth_execute, instantiate as mock_pyth_instantiate,
        query as mock_pyth_query,
    };
    use mock_pyth::msg::ExecuteMsg as MockPythExecuteMsg;
    use oracle::contract::{
        execute as oracle_execute, instantiate as oracle_instantiate, query as oracle_query,
    };
    use oracle::msg::InstantiateMsg as OracleInstantiateMsg;
    use std::collections::HashMap;

    const OWNER: &str = "neutron17ykn47jnxnn83ceh95grafvtjx7xzsstw0jq9d";
    const LIQUIDATOR: &str = "neutron1utl04swwlt9xsr9c2vdx2nkea4plp66sx3s8pg";
    const LIQ_THRESHOLD: Uint128 = Uint128::new(50);
    const LIQ_BONUS: Uint128 = Uint128::new(10);
    const MIN_HEALTH_FACTOR: Decimal = Decimal::one();
    const NATIVE_COLLATERAL_DENOM: &str = "native";
    const INITIAL_OWNER_NATIVE_BALANCE: u128 = 15_000_000;
    const CW20_COLLATERAL_DENOM: &str = "address";
    const CW20_AMOUNT_MINTED_TO_OWNER: Uint128 = Uint128::new(1_000_000_000_000);
    const ORACLE_ADDRESS: &str = "oracle_addr";
    const PYTH_ORACLE_ADDRESS: &str = "pyth_oracle_addr";
    const PRICE_FEED_ID_1: &str =
        "63f341689d98a12ef60a5cff1d7f85c70a9e17bf1575f0e7c0b2512d48b1c8b3";
    const PRICE_FEED_ID_2: &str =
        "2b9ab1e972a281585084148ba1389800799bd4be63b957507db1349314e47445";
    const DSC_ADDR: &str = "dsc_addr";
    const AMOUNT_COLLATERAL_OK: Uint128 = Uint128::new(2_000_000); // 2_000_000/1_000_000 * 680_000/100_000 = 13.6 usd at mock oracle price
    const AMOUNT_DSC_TO_MINT_OK: Uint128 = Uint128::new(1_000_000); // health factor = 6.8
    const DEBT_TO_COVER: Uint128 = Uint128::new(900_000);
    const LIQUIDATION_PRICE: i64 = 97_000;
    const FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR: Uint128 = Uint128::new(2_000_000); // Same as initial
    const FINAL_COLLATERAL_BALANCE_OF_LIQUIDATED: Uint128 = Uint128::new(979_382); // 2_000_000/1_000_000 - (900_000/1_000_000 * 100_000/97_000) * 1.1
    const FINAL_DSC_BALANCE_OF_LIQUIDATOR: Uint128 = Uint128::new(100_000); // 1_000_000 - 900_000
    const FINAL_DSC_BALANCE_OF_LIQUIDATED: Uint128 = Uint128::new(1_000_000); // Same as initial
    const FINAL_DSC_SUPPLY: Uint128 = Uint128::new(1_100_000); // 2_000_000 - 900_000
    const FINAL_NATIVE_BALANCE_OF_LIQUIDATOR: Uint128 = Uint128::new(1_020_618); // (900_000/1_000_000 * 100_000/97_000) * 1.1
    const FINAL_NATIVE_BALANCE_OF_LIQUIDATED: Uint128 = Uint128::new(11_000_000); // 15_000_000 - 2_000_000 - 2_000_000

    fn get_default_instantiate_msg(
        cw20_address: Option<&str>,
        dsc_address: Option<&str>,
        oracle_address: Option<&str>,
        pyth_oracle_address: Option<&str>,
    ) -> InstantiateMsg {
        let native_collateral: AssetInfo = AssetInfo::Native(NATIVE_COLLATERAL_DENOM.to_string());
        let cw20_denom = cw20_address.unwrap_or(CW20_COLLATERAL_DENOM);
        let cw20_collateral: AssetInfo = AssetInfo::Cw20(Addr::unchecked(cw20_denom.to_string()));
        // let assets: Vec<AssetInfo> = vec![native_collateral, cw20_collateral];
        let assets: Vec<AssetInfo> = vec![native_collateral, cw20_collateral];
        let dsc_address = dsc_address.unwrap_or(DSC_ADDR);
        let oracle_address = oracle_address.unwrap_or(ORACLE_ADDRESS);
        let pyth_oracle_address = pyth_oracle_address.unwrap_or(PYTH_ORACLE_ADDRESS);
        let price_feed_ids: Vec<String> =
            vec![String::from(PRICE_FEED_ID_1), String::from(PRICE_FEED_ID_2)];
        InstantiateMsg {
            owner: String::from(OWNER),
            assets: assets,
            oracle_address: String::from(oracle_address),
            pyth_oracle_address: String::from(pyth_oracle_address),
            price_feed_ids: price_feed_ids,
            dsc_address: String::from(dsc_address),
            liquidation_threshold: LIQ_THRESHOLD,
            liquidation_bonus: LIQ_BONUS,
            min_health_factor: MIN_HEALTH_FACTOR,
        }
    }

    fn get_cw20_instantiate_msg() -> Cw20InstantiateMsg {
        Cw20InstantiateMsg {
            name: String::from("CW20 Token"),
            symbol: String::from(CW20_COLLATERAL_DENOM),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: String::from(OWNER),
                amount: CW20_AMOUNT_MINTED_TO_OWNER,
            }],
            mint: None,
            marketing: None,
        }
    }

    fn get_dsc_instantiate_msg() -> Cw20InstantiateMsg {
        Cw20InstantiateMsg {
            name: String::from("Decentralized Stablecoin"),
            symbol: String::from("DSC"),
            decimals: 6,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: String::from(OWNER),
                cap: None,
            }),
            marketing: None,
        }
    }

    fn dsc_engine_setup(cw20_address: Option<&str>) -> (App, Addr) {
        let mut app = App::default();
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id: u64 = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &get_default_instantiate_msg(cw20_address, None, None, None),
                &[],
                "dsc_engine",
                Some(String::from(OWNER)),
            )
            .unwrap();
        (app, addr)
    }

    #[test]
    fn proper_instantiation() {
        let (app, addr) = dsc_engine_setup(None);

        let config_input = get_default_instantiate_msg(None, None, None, None);

        let config_res: ConfigResponse = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::Config {})
            .unwrap();

        let assets_to_feeds: HashMap<String, String> = config_input
            .assets
            .into_iter()
            .map(|asset| asset.to_string())
            .zip(config_input.price_feed_ids.into_iter())
            .collect();

        assert_eq!(&config_res.owner, &config_input.owner);
        assert_eq!(config_res.assets_to_feeds, assets_to_feeds);
        assert_eq!(&config_res.oracle_address, &config_input.oracle_address);
        assert_eq!(&config_res.dsc_address, &config_input.dsc_address);
    }

    #[test]
    fn proper_deposit_valid_cw20_collateral_and_mint() {
        let mut app = App::default();

        // 1 - Instantiate mock-pyth and price oracle

        let mock_pyth_code =
            ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
        let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

        let mock_pyth_price_feed_addr = app
            .instantiate_contract(
                mock_pyth_code_id,
                Addr::unchecked(OWNER),
                &Empty {},
                &[],
                "mock-pyth",
                Some(String::from(OWNER)),
            )
            .unwrap();

        let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
        let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

        let oracle_addr = app
            .instantiate_contract(
                oracle_code_id,
                Addr::unchecked(OWNER),
                &OracleInstantiateMsg {},
                &[],
                "oracle",
                Some(String::from(OWNER)),
            )
            .unwrap();

        let price: FetchPriceResponse = app
            .wrap()
            .query_wasm_smart(
                oracle_addr.clone(),
                &OracleQueryMsg::FetchPrice {
                    pyth_contract_addr: mock_pyth_price_feed_addr.to_string(),
                    price_feed_id: PriceIdentifier::from_hex(PRICE_FEED_ID_1).unwrap(),
                },
            )
            .unwrap();

        // println!("PRICE FEED ADDR:");
        // println!("{:?}", mock_pyth_price_feed_addr.to_string());
        // println!();
        // println!("PRICE:");
        // println!("{:?}", price);

        // 2 - Instantiate cw20 token contract and mint to test user at instantiation

        let cw20_code = ContractWrapper::new(cw20_execute, cw20_instantiate, cw20_query);
        let cw20_code_id: u64 = app.store_code(Box::new(cw20_code));

        let cw20_addr = app
            .instantiate_contract(
                cw20_code_id,
                Addr::unchecked(OWNER),
                &get_cw20_instantiate_msg(),
                &[],
                "cw20",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 3 - Instantiate DSC

        let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
        let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

        let dsc_addr = app
            .instantiate_contract(
                dsc_code_id,
                Addr::unchecked(OWNER),
                &get_dsc_instantiate_msg(),
                &[],
                "dsc",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 4 - Instantiate dsce contract

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id: u64 = app.store_code(Box::new(code));

        let dsce_init_msg = &get_default_instantiate_msg(
            Some(cw20_addr.as_str()),
            Some(dsc_addr.as_str()),
            Some(oracle_addr.as_str()),
            Some(mock_pyth_price_feed_addr.as_str()),
        );

        // println!("INIT MSG");
        // println!("{:?}", dsce_init_msg);

        let dsce_addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &dsce_init_msg,
                &[],
                "dsc_engine",
                Some(String::from(OWNER)),
            )
            .unwrap();

        let config_res: ConfigResponse = app
            .wrap()
            .query_wasm_smart(dsce_addr.clone(), &QueryMsg::Config {})
            .unwrap();

        // println!("CONFIG:");
        // println!("{:?}", config_res);

        // 5 - Update dsc minter to dsce

        let update_dsc_minter_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsc_addr.clone(),
                &Cw20ExecuteMsg::UpdateMinter {
                    new_minter: Some(String::from(dsce_addr.clone())),
                },
                &[],
            )
            .unwrap();

        // 6 - Increase Allowance of cw20 from user to dsce contract

        let increase_allowance_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                cw20_addr.clone(),
                &Cw20ExecuteMsg::IncreaseAllowance {
                    spender: String::from(dsce_addr.clone()),
                    amount: CW20_AMOUNT_MINTED_TO_OWNER,
                    expires: None,
                },
                &[],
            )
            .unwrap();

        // 7 - execute deposit_collateral_and_mint_dsc

        let initial_deposited_owner_cw20_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr.clone(),
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(cw20_addr.clone()),
                },
            )
            .unwrap();

        let resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsce_addr.clone(),
                &ExecuteMsg::DepositCollateralAndMintDsc {
                    collateral_asset: AssetInfo::Cw20(Addr::unchecked(cw20_addr.as_str())),
                    amount_collateral: AMOUNT_COLLATERAL_OK,
                    amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
                },
                &[],
            )
            .unwrap();
        // println!("RESP: {:?}", resp);

        // Test deposit events
        let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "action")
                .unwrap()
                .value,
            "deposit_collateral"
        );
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "from")
                .unwrap()
                .value,
            OWNER
        );
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "asset")
                .unwrap()
                .value,
            cw20_addr.as_str()
        );
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "amount")
                .unwrap()
                .value,
            String::from(AMOUNT_COLLATERAL_OK)
        );

        let final_deposited_owner_cw20_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr,
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(cw20_addr),
                },
            )
            .unwrap();

        let final_owner_dsc_balance: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                dsc_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: String::from(OWNER),
                },
            )
            .unwrap();

        let final_dsc_info: TokenInfoResponse = app
            .wrap()
            .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
            .unwrap();

        assert_eq!(initial_deposited_owner_cw20_balance, Uint128::zero());
        assert_eq!(final_deposited_owner_cw20_balance, AMOUNT_COLLATERAL_OK);
        assert_eq!(final_dsc_info.total_supply, AMOUNT_DSC_TO_MINT_OK);
        assert_eq!(final_owner_dsc_balance.balance, AMOUNT_DSC_TO_MINT_OK);
    }

    #[test]
    fn proper_deposit_valid_native_collateral_and_mint() {
        // 0 - Instantiate mock env with native balances associated to OWNER
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(OWNER),
                    coins(INITIAL_OWNER_NATIVE_BALANCE, NATIVE_COLLATERAL_DENOM),
                )
                .unwrap()
        });

        // 1 - Instantiate mock-pyth and price oracle

        let mock_pyth_code =
            ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
        let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

        let mock_pyth_price_feed_addr = app
            .instantiate_contract(
                mock_pyth_code_id,
                Addr::unchecked(OWNER),
                &Empty {},
                &[],
                "mock-pyth",
                Some(String::from(OWNER)),
            )
            .unwrap();

        let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
        let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

        let oracle_addr = app
            .instantiate_contract(
                oracle_code_id,
                Addr::unchecked(OWNER),
                &OracleInstantiateMsg {},
                &[],
                "oracle",
                Some(String::from(OWNER)),
            )
            .unwrap();

        let price: FetchPriceResponse = app
            .wrap()
            .query_wasm_smart(
                oracle_addr.clone(),
                &OracleQueryMsg::FetchPrice {
                    pyth_contract_addr: mock_pyth_price_feed_addr.to_string(),
                    price_feed_id: PriceIdentifier::from_hex(PRICE_FEED_ID_1).unwrap(),
                },
            )
            .unwrap();

        // 2 - Instantiate DSC

        let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
        let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

        let dsc_addr = app
            .instantiate_contract(
                dsc_code_id,
                Addr::unchecked(OWNER),
                &get_dsc_instantiate_msg(),
                &[],
                "dsc",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 3 - Instantiate dsce contract

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id: u64 = app.store_code(Box::new(code));

        let dsce_init_msg = &get_default_instantiate_msg(
            None,
            Some(dsc_addr.as_str()),
            Some(oracle_addr.as_str()),
            Some(mock_pyth_price_feed_addr.as_str()),
        );

        let dsce_addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &dsce_init_msg,
                &[],
                "dsc_engine",
                Some(String::from(OWNER)),
            )
            .unwrap();

        let config_res: ConfigResponse = app
            .wrap()
            .query_wasm_smart(dsce_addr.clone(), &QueryMsg::Config {})
            .unwrap();

        // 4 - Update dsc minter to dsce

        let update_dsc_minter_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsc_addr.clone(),
                &Cw20ExecuteMsg::UpdateMinter {
                    new_minter: Some(String::from(dsce_addr.clone())),
                },
                &[],
            )
            .unwrap();

        // 5 - execute deposit_collateral_and_mint_dsc

        let initial_deposited_owner_native_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr.clone(),
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
                },
            )
            .unwrap();

        let resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsce_addr.clone(),
                &ExecuteMsg::DepositCollateralAndMintDsc {
                    collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                    amount_collateral: AMOUNT_COLLATERAL_OK,
                    amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
                },
                &[Coin {
                    denom: String::from(NATIVE_COLLATERAL_DENOM),
                    amount: AMOUNT_COLLATERAL_OK,
                }],
            )
            .unwrap();

        let final_deposited_owner_native_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr.clone(),
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
                },
            )
            .unwrap();

        let final_owner_dsc_balance: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                dsc_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: String::from(OWNER),
                },
            )
            .unwrap();

        let final_dsc_info: TokenInfoResponse = app
            .wrap()
            .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
            .unwrap();

        assert_eq!(initial_deposited_owner_native_balance, Uint128::zero());
        assert_eq!(
            final_deposited_owner_native_balance,
            Uint128::from(AMOUNT_COLLATERAL_OK)
        );
        assert_eq!(final_dsc_info.total_supply, AMOUNT_DSC_TO_MINT_OK);
        assert_eq!(final_owner_dsc_balance.balance, AMOUNT_DSC_TO_MINT_OK);
    }

    #[test]
    fn proper_redeem_valid_cw20_collateral_and_burn_dsc() {
        let mut app = App::default();

        // 1 - Instantiate mock-pyth and price oracle

        let mock_pyth_code =
            ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
        let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

        let mock_pyth_price_feed_addr = app
            .instantiate_contract(
                mock_pyth_code_id,
                Addr::unchecked(OWNER),
                &Empty {},
                &[],
                "mock-pyth",
                Some(String::from(OWNER)),
            )
            .unwrap();

        let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
        let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

        let oracle_addr = app
            .instantiate_contract(
                oracle_code_id,
                Addr::unchecked(OWNER),
                &OracleInstantiateMsg {},
                &[],
                "oracle",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 2 - Instantiate cw20 token contract and mint to test user at instantiation

        let cw20_code = ContractWrapper::new(cw20_execute, cw20_instantiate, cw20_query);
        let cw20_code_id: u64 = app.store_code(Box::new(cw20_code));

        let cw20_addr = app
            .instantiate_contract(
                cw20_code_id,
                Addr::unchecked(OWNER),
                &get_cw20_instantiate_msg(),
                &[],
                "cw20",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 3 - Instantiate DSC

        let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
        let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

        let dsc_addr = app
            .instantiate_contract(
                dsc_code_id,
                Addr::unchecked(OWNER),
                &get_dsc_instantiate_msg(),
                &[],
                "dsc",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 4 - Instantiate dsce contract

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id: u64 = app.store_code(Box::new(code));

        let dsce_init_msg = &get_default_instantiate_msg(
            Some(cw20_addr.as_str()),
            Some(dsc_addr.as_str()),
            Some(oracle_addr.as_str()),
            Some(mock_pyth_price_feed_addr.as_str()),
        );

        let dsce_addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &dsce_init_msg,
                &[],
                "dsc_engine",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 5 - Update dsc minter to dsce

        let update_dsc_minter_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsc_addr.clone(),
                &Cw20ExecuteMsg::UpdateMinter {
                    new_minter: Some(String::from(dsce_addr.clone())),
                },
                &[],
            )
            .unwrap();

        // 6 - Increase Allowance of cw20 from user to dsce contract

        let increase_allowance_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                cw20_addr.clone(),
                &Cw20ExecuteMsg::IncreaseAllowance {
                    spender: String::from(dsce_addr.clone()),
                    amount: CW20_AMOUNT_MINTED_TO_OWNER,
                    expires: None,
                },
                &[],
            )
            .unwrap();

        // 7 - execute deposit_collateral_and_mint_dsc

        let deposit_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsce_addr.clone(),
                &ExecuteMsg::DepositCollateralAndMintDsc {
                    collateral_asset: AssetInfo::Cw20(Addr::unchecked(cw20_addr.as_str())),
                    amount_collateral: AMOUNT_COLLATERAL_OK,
                    amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
                },
                &[],
            )
            .unwrap();

        // 8 - Increase Allowance of dsc (cw20) from user to dsce contract

        let increase_allowance_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsc_addr.clone(),
                &Cw20ExecuteMsg::IncreaseAllowance {
                    spender: String::from(dsce_addr.clone()),
                    amount: AMOUNT_DSC_TO_MINT_OK,
                    expires: None,
                },
                &[],
            )
            .unwrap();

        // 9 - execute reddem_collateral_and_burn_dsc

        let initial_deposited_owner_cw20_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr.clone(),
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(cw20_addr.clone()),
                },
            )
            .unwrap();

        let resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsce_addr.clone(),
                &ExecuteMsg::RedeemCollateralForDsc {
                    collateral_asset: AssetInfo::Cw20(Addr::unchecked(cw20_addr.as_str())),
                    amount_collateral: AMOUNT_COLLATERAL_OK,
                    amount_dsc_to_burn: AMOUNT_DSC_TO_MINT_OK,
                },
                &[],
            )
            .unwrap();

        // Test deposit events
        let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "action")
                .unwrap()
                .value,
            "redeem_collateral"
        );
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "from")
                .unwrap()
                .value,
            OWNER
        );
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "asset")
                .unwrap()
                .value,
            cw20_addr.as_str()
        );
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "amount")
                .unwrap()
                .value,
            String::from(AMOUNT_COLLATERAL_OK)
        );

        let final_deposited_owner_cw20_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr,
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(cw20_addr),
                },
            )
            .unwrap();

        let final_owner_dsc_balance: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                dsc_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: String::from(OWNER),
                },
            )
            .unwrap();

        let final_dsc_info: TokenInfoResponse = app
            .wrap()
            .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
            .unwrap();

        assert_eq!(initial_deposited_owner_cw20_balance, AMOUNT_COLLATERAL_OK);
        assert_eq!(final_deposited_owner_cw20_balance, Uint128::zero());
        assert_eq!(final_dsc_info.total_supply, Uint128::zero());
        assert_eq!(final_owner_dsc_balance.balance, Uint128::zero());
    }

    #[test]
    fn proper_redeem_valid_native_collateral_and_burn_dsc() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(OWNER),
                    coins(INITIAL_OWNER_NATIVE_BALANCE, NATIVE_COLLATERAL_DENOM),
                )
                .unwrap()
        });

        // 1 - Instantiate mock-pyth and price oracle

        let mock_pyth_code =
            ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
        let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

        let mock_pyth_price_feed_addr = app
            .instantiate_contract(
                mock_pyth_code_id,
                Addr::unchecked(OWNER),
                &Empty {},
                &[],
                "mock-pyth",
                Some(String::from(OWNER)),
            )
            .unwrap();

        let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
        let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

        let oracle_addr = app
            .instantiate_contract(
                oracle_code_id,
                Addr::unchecked(OWNER),
                &OracleInstantiateMsg {},
                &[],
                "oracle",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 2 - Instantiate DSC

        let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
        let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

        let dsc_addr = app
            .instantiate_contract(
                dsc_code_id,
                Addr::unchecked(OWNER),
                &get_dsc_instantiate_msg(),
                &[],
                "dsc",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 3 - Instantiate dsce contract

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id: u64 = app.store_code(Box::new(code));

        let dsce_init_msg = &get_default_instantiate_msg(
            None,
            Some(dsc_addr.as_str()),
            Some(oracle_addr.as_str()),
            Some(mock_pyth_price_feed_addr.as_str()),
        );

        let dsce_addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &dsce_init_msg,
                &[],
                "dsc_engine",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 4 - Update dsc minter to dsce

        let update_dsc_minter_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsc_addr.clone(),
                &Cw20ExecuteMsg::UpdateMinter {
                    new_minter: Some(String::from(dsce_addr.clone())),
                },
                &[],
            )
            .unwrap();

        // 5 - execute deposit_collateral_and_mint_dsc

        let deposit_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsce_addr.clone(),
                &ExecuteMsg::DepositCollateralAndMintDsc {
                    collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                    amount_collateral: AMOUNT_COLLATERAL_OK,
                    amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
                },
                &[Coin {
                    denom: String::from(NATIVE_COLLATERAL_DENOM),
                    amount: AMOUNT_COLLATERAL_OK,
                }],
            )
            .unwrap();

        let initial_deposited_owner_native_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr.clone(),
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
                },
            )
            .unwrap();

        // 6 - Increase Allowance of dsc (cw20) from user to dsce contract

        let increase_allowance_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsc_addr.clone(),
                &Cw20ExecuteMsg::IncreaseAllowance {
                    spender: String::from(dsce_addr.clone()),
                    amount: AMOUNT_DSC_TO_MINT_OK,
                    expires: None,
                },
                &[],
            )
            .unwrap();

        // 7 - execute redeem_collateral_and_burn_dsc

        let resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsce_addr.clone(),
                &ExecuteMsg::RedeemCollateralForDsc {
                    collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                    amount_collateral: AMOUNT_COLLATERAL_OK,
                    amount_dsc_to_burn: AMOUNT_DSC_TO_MINT_OK,
                },
                &[],
            )
            .unwrap();

        let final_deposited_owner_native_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr.clone(),
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
                },
            )
            .unwrap();

        let final_owner_dsc_balance: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                dsc_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: String::from(OWNER),
                },
            )
            .unwrap();

        let final_dsc_info: TokenInfoResponse = app
            .wrap()
            .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
            .unwrap();

        assert_eq!(initial_deposited_owner_native_balance, AMOUNT_COLLATERAL_OK);
        assert_eq!(final_deposited_owner_native_balance, Uint128::zero());
        assert_eq!(final_dsc_info.total_supply, Uint128::zero());
        assert_eq!(final_owner_dsc_balance.balance, Uint128::zero());
    }

    #[test]
    fn proper_native_liquidation() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(OWNER),
                    coins(INITIAL_OWNER_NATIVE_BALANCE, NATIVE_COLLATERAL_DENOM),
                )
                .unwrap()
        });

        let send_msg = app.send_tokens(
            Addr::unchecked(OWNER),
            Addr::unchecked(LIQUIDATOR),
            &[Coin {
                denom: String::from(NATIVE_COLLATERAL_DENOM),
                amount: AMOUNT_COLLATERAL_OK,
            }],
        );

        // 1 - Instantiate mock-pyth and price oracle

        let mock_pyth_code =
            ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
        let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

        let mock_pyth_price_feed_addr = app
            .instantiate_contract(
                mock_pyth_code_id,
                Addr::unchecked(OWNER),
                &Empty {},
                &[],
                "mock-pyth",
                Some(String::from(OWNER)),
            )
            .unwrap();

        let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
        let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

        let oracle_addr = app
            .instantiate_contract(
                oracle_code_id,
                Addr::unchecked(OWNER),
                &OracleInstantiateMsg {},
                &[],
                "oracle",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 2 - Instantiate DSC

        let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
        let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

        let dsc_addr = app
            .instantiate_contract(
                dsc_code_id,
                Addr::unchecked(OWNER),
                &get_dsc_instantiate_msg(),
                &[],
                "dsc",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 3 - Instantiate dsce contract

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id: u64 = app.store_code(Box::new(code));

        let dsce_init_msg = &get_default_instantiate_msg(
            None,
            Some(dsc_addr.as_str()),
            Some(oracle_addr.as_str()),
            Some(mock_pyth_price_feed_addr.as_str()),
        );

        let dsce_addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &dsce_init_msg,
                &[],
                "dsc_engine",
                Some(String::from(OWNER)),
            )
            .unwrap();

        // 4 - Update dsc minter to dsce

        let update_dsc_minter_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsc_addr.clone(),
                &Cw20ExecuteMsg::UpdateMinter {
                    new_minter: Some(String::from(dsce_addr.clone())),
                },
                &[],
            )
            .unwrap();

        // 5 - execute deposit_collateral_and_mint_dsc by user that will be liquidated

        let deposit_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                dsce_addr.clone(),
                &ExecuteMsg::DepositCollateralAndMintDsc {
                    collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                    amount_collateral: AMOUNT_COLLATERAL_OK,
                    amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
                },
                &[Coin {
                    denom: String::from(NATIVE_COLLATERAL_DENOM),
                    amount: AMOUNT_COLLATERAL_OK,
                }],
            )
            .unwrap();

        let initial_deposited_owner_native_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr.clone(),
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
                },
            )
            .unwrap();

        // 6 - execute deposit_collateral_and_mint_dsc by liquidator

        let liquidator_deposit_resp = app
            .execute_contract(
                Addr::unchecked(LIQUIDATOR),
                dsce_addr.clone(),
                &ExecuteMsg::DepositCollateralAndMintDsc {
                    collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                    amount_collateral: AMOUNT_COLLATERAL_OK,
                    amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
                },
                &[Coin {
                    denom: String::from(NATIVE_COLLATERAL_DENOM),
                    amount: AMOUNT_COLLATERAL_OK,
                }],
            )
            .unwrap();

        // 6 - Increase Allowance of dsc (cw20) from liquidator to dsce contract

        let increase_allowance_resp = app
            .execute_contract(
                Addr::unchecked(LIQUIDATOR),
                dsc_addr.clone(),
                &Cw20ExecuteMsg::IncreaseAllowance {
                    spender: String::from(dsce_addr.clone()),
                    amount: AMOUNT_DSC_TO_MINT_OK,
                    expires: None,
                },
                &[],
            )
            .unwrap();

        // Change mock-pyth collateral price - set new lower mocked price

        let update_mock_price_resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                mock_pyth_price_feed_addr.clone(),
                &MockPythExecuteMsg::UpdateMockPrice {
                    price: LIQUIDATION_PRICE,
                },
                &[],
            )
            .unwrap();

        // liquidate

        // panicked at 'attempt to subtract with overflow'
        // OWNER DSC OR DEPOSITED BALANCES ARE ZERO?

        let liquidation_resp = app
            .execute_contract(
                Addr::unchecked(LIQUIDATOR),
                dsce_addr.clone(),
                &ExecuteMsg::Liquidate {
                    collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                    user: String::from(OWNER),
                    debt_to_cover: Decimal::from_atomics(DEBT_TO_COVER, 6).unwrap(),
                },
                &[],
            )
            .unwrap();

        // println!("LIQUIDATION RESPONSE: {:?}", liquidation_resp);

        let final_deposited_owner_native_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr.clone(),
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(OWNER),
                    collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
                },
            )
            .unwrap();

        let final_deposited_liquidator_native_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                dsce_addr.clone(),
                &QueryMsg::CollateralBalanceOfUser {
                    user: String::from(LIQUIDATOR),
                    collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
                },
            )
            .unwrap();

        let final_owner_dsc_balance: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                dsc_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: String::from(OWNER),
                },
            )
            .unwrap();

        let final_liquidator_dsc_balance: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                dsc_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: String::from(LIQUIDATOR),
                },
            )
            .unwrap();

        let final_owner_native_balance: Uint128 = app
            .wrap()
            .query_balance(OWNER, NATIVE_COLLATERAL_DENOM)
            .unwrap()
            .amount;

        let final_liquidator_native_balance: Uint128 = app
            .wrap()
            .query_balance(LIQUIDATOR, NATIVE_COLLATERAL_DENOM)
            .unwrap()
            .amount;

        let final_dsc_info: TokenInfoResponse = app
            .wrap()
            .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
            .unwrap();

        assert_eq!(
            final_deposited_owner_native_balance,
            FINAL_COLLATERAL_BALANCE_OF_LIQUIDATED
        );
        assert_eq!(
            final_deposited_liquidator_native_balance,
            FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR
        );
        assert_eq!(
            final_owner_dsc_balance.balance,
            FINAL_DSC_BALANCE_OF_LIQUIDATED
        );
        assert_eq!(
            final_liquidator_dsc_balance.balance,
            FINAL_DSC_BALANCE_OF_LIQUIDATOR
        );
        assert_eq!(
            final_owner_native_balance,
            FINAL_NATIVE_BALANCE_OF_LIQUIDATED
        );
        assert_eq!(
            final_liquidator_native_balance,
            FINAL_NATIVE_BALANCE_OF_LIQUIDATOR
        );
        assert_eq!(final_dsc_info.total_supply, FINAL_DSC_SUPPLY);
    }
}
