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

use crate::msg::{
    ExecuteMsg,
    FetchPriceResponse,
    InstantiateMsg,
    MigrateMsg,
    QueryMsg,
};
use crate::state::{
    Oracle,
    State,
    STATE,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "migrate"))
}

/// The instantiate function is invoked when the contract is first deployed. This function sets
/// configuration values that affect how the contract behaves. It is possible for deployers of
/// this contract to set the Oracle here. In this example, we provide two oracles:
///
/// - Pyth (An address of a real on-chain contract to interact with)
/// - Stub (An example Oracle that can be used for testing).
///
/// The Stub Oracle uses Pyth datastructures to simulate Pyth behaviour, see the tests in this file
/// to see an example of how to write contract tests that rely on a stubbed Pyth oracle to model
/// different simulated price oracle behaviours.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = match msg {
        InstantiateMsg::StubOracle { maybe_price } => State {
            oracle: Oracle::Stub(maybe_price),
        },
        InstantiateMsg::PythOracle {
            ref contract_addr,
            price_id,
        } => State {
            oracle: Oracle::Pyth(deps.api.addr_validate(contract_addr.as_ref())?, price_id),
        },
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("price_id", format!("{:?}", msg)))
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
        QueryMsg::FetchPrice {} => to_binary(&query_fetch_price(deps)?),
    }
}

/// Allow the caller to query the current (most recent) price the Oracle has observed, which is
/// stored in the contract state.
fn query_fetch_price(deps: Deps) -> StdResult<FetchPriceResponse> {
    let state = STATE.load(deps.storage)?;

    let price = state
        .oracle
        .get_price(&deps.querier)
        .ok_or_else(|| StdError::not_found("Current price is not available"))?;

    Ok(FetchPriceResponse { price })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::DepsMut;
    use pyth_sdk_terra::Price;

    use crate::msg::FetchPriceResponse;
    use crate::state::{
        Oracle,
        State,
        STATE,
    };

    use super::query_fetch_price;

    /// set_price provides a helper that mutates the terra state for the example contract. This can
    /// be used to make modifications to the state before invoking the contract itself. This is
    /// helpful for testing contract behaviour with various states.
    pub fn set_price(deps: DepsMut, maybe_price: Option<Price>) {
        STATE
            .save(
                deps.storage,
                &State {
                    oracle: Oracle::Stub(maybe_price),
                },
            )
            .unwrap();
    }

    /// Quick test to confirm that after calling set_price, querying the contract state produces
    /// the new price.
    #[test]
    pub fn test_query_fetch_price_ok() {
        let mut deps = mock_dependencies(&[]);

        let price = Price {
            price: 1000,
            conf:  5,
            expo:  0,
        };

        set_price(deps.as_mut(), Some(price));

        assert_eq!(
            query_fetch_price(deps.as_ref()),
            Ok(FetchPriceResponse { price })
        );
    }

    /// Quick test to make sure that when removing any price, the query fails.
    #[test]
    pub fn test_query_fetch_price_unavailable() {
        let mut deps = mock_dependencies(&[]);

        set_price(deps.as_mut(), None);

        assert!(query_fetch_price(deps.as_ref()).is_err());
    }
}
