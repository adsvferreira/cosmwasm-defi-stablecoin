use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Empty, Env, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};
#[cfg(not(feature = "library"))]
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use cw_asset::AssetInfo;

use crate::error::ContractError;
use crate::msg::{AccountInfoResponse, ExecuteMsg, InstantiateMsg};
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
        // CHECK IF COLLATERAL ASSET IS VALID
        let config = CONFIG.load(deps.storage)?;
        if !config
            .assets_to_feeds
            .contains_key(&collateral_asset.to_string())
        {
            return Err(ContractError::InvalidCollateralAsset {
                denom: collateral_asset.inner(),
            });
        }

        // TRANSFER COLLATERAL FROM USER TO CONTRACT
        // If the asset is a token contract, then we need to execute a TransferFrom msg to receive assets
        // If the asset is native token, the pool balance is already increased
        let mut messages: std::vec::Vec<CosmosMsg<Empty>> = vec![];
        if let AssetInfo::Cw20(contract_addr) = &collateral_asset {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
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

        // MINT DSC TO USER
        // NOTE: DSC Engine must be declared as minter on DSC CW20 intantiation
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.dsc_address.into_string(),
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
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

        // VERIFY NEW USER HEALTH FACTOR
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

        // BURN DSC
        // NOTE: DSC Engine must be declared as minter on DSC CW20 intantiation
        let burn_dsc_msg = _burn_dsc(
            deps.storage,
            &env,
            amount_dsc_to_burn,
            &info.sender,
            &info.sender,
        )?;
        messages.push(burn_dsc_msg);

        // REDEEM COLLATERAL
        let redeem_collateral_msg = _redeem_collateral(
            deps.storage,
            &collateral_asset,
            amount_collateral,
            &info.sender,
            &info.sender,
        )?;
        messages.push(redeem_collateral_msg);

        // VERIFY NEW USER HEALTH FACTOR
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

        // REDEEM COLLATERAL
        let redeem_collateral_msg = _redeem_collateral(
            deps.storage,
            &collateral_asset,
            precision_adjusted_collateral_to_redeem,
            user_addr,
            &info.sender,
        )?;
        messages.push(redeem_collateral_msg);

        // BURN DSC
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
        let burn_dsc_msg = _burn_dsc(
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
                msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
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
            msg: to_json_binary(&Cw20ExecuteMsg::BurnFrom {
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
            msg: to_json_binary(&OracleQueryMsg::FetchPrice {
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
