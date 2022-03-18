#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary,
    Binary,
    Decimal,
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
    Ok(Response::new().add_attribute("method", "instantiate"))
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

    match price_feed.get_current_price() {
        Some(current_price) => {
            let price = if current_price.expo < 0 {
                Decimal::from_ratio(
                    current_price.price as u128,
                    10u128.pow((-current_price.expo) as u32),
                )
            } else {
                Decimal::from_ratio(
                    (current_price.price as u128) * 10u128.pow(current_price.expo as u32),
                    1u128,
                )
            };

            Ok(FetchPriceResponse { price })
        }
        None => Err(StdError::generic_err("Current price is not available")),
    }
}
