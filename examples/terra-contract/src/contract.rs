#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary,
    Binary,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
};

use pyth_sdk_terra::query_price_feed;

use crate::msg::{
    ExecuteMsg,
    FetchPriceResponse,
    InstantiateMsg,
    MigrateMsg,
    QueryMsg,
};
use crate::state::{
    State,
    STATE,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "migrate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // It is a good practice that your contract stores the pyth contract address and ids of the price feeds
    // it needs upon instantiation or by an authorized approach. This will ensure that a wrong address won't
    // be used.
    let state = State {
        pyth_contract_addr: deps.api.addr_validate(msg.pyth_contract_addr.as_ref())?,
        price_feed_id:      msg.price_feed_id,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("price_id", format!("{:#x?}", msg.price_feed_id)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "execute"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::FetchPrice => to_binary(&query_fetch_price(deps)?),
    }
}

fn query_fetch_price(deps: Deps) -> StdResult<FetchPriceResponse> {
    let state = STATE.load(deps.storage)?;

    let price_feed = query_price_feed(
        &deps.querier,
        state.pyth_contract_addr.into_string(),
        state.price_feed_id,
    )?
    .price_feed;

    // This examples throws an error if the price is not available. Price could be
    // unavailable if the number of publishers are low or it has not been updated
    // for a while due to network errors and etc. It is recommended that you handle
    // the scenarios which price is not available in a better way.
    // Make sure to read [consumer best practices](https://docs.pyth.network/consumers/best-practices)
    // when using pyth data.
    let current_price = price_feed
        .get_current_price()
        .ok_or(StdError::not_found("Current price is not available"))?;
    let ema_price = price_feed
        .get_ema_price()
        .ok_or(StdError::not_found("EMA price is not available"))?;

    return Ok(FetchPriceResponse {
        current_price,
        ema_price,
    });
}
