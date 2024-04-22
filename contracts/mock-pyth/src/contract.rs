use crate::error::ContractError;
use crate::msg::ExecuteMsg;
use crate::state::PRICE;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use pyth_sdk_cw::{Price, PriceFeed, PriceFeedResponse, PriceIdentifier, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateMockPrice { price } => update_mock_price(deps, price),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PriceFeed { id } => to_json_binary(&mocked_price_feed(deps, id)?),
        _ => panic!("Unsupported query!"),
    }
}

fn mocked_price_feed(deps: Deps, id: PriceIdentifier) -> StdResult<PriceFeedResponse> {
    let default_price = 680000;
    let state_price = PRICE.may_load(deps.storage)?;
    let price = match state_price {
        Some(price) => price,
        None => default_price,
    };
    let ema_price = price + 100;

    let price_feed_response = PriceFeedResponse {
        price_feed: PriceFeed::new(
            id,
            Price {
                price: price,
                conf: 510000,
                expo: -5,
                publish_time: 1571797419,
            },
            Price {
                price: ema_price,
                conf: 400000,
                expo: -5,
                publish_time: 1571797419,
            },
        ),
    };

    Ok(price_feed_response)
}

fn update_mock_price(deps: DepsMut, price: i64) -> Result<Response, ContractError> {
    PRICE.save(deps.storage, &price)?;
    Ok(Response::default())
}
