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

/// The instantiate function is invoked when the contract is first deployed. We use this function
/// to set configuration that affect how the contract behaves. In this example, it is possible for
/// deployers of this contract to choose an Oracle. We provide two options:
///
/// - Pyth (An address of a real on-chain Pyth contract to interact with)
/// - Stub (An example Oracle that can be used for testing).
///
/// The Stub Oracle uses Pyth datastructures to simulate Pyth behaviour. See the tests in this file
/// to see an example of how to write contract tests that rely on the stubbed Pyth oracle to model
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

/// Allow the caller to query the current (most recent) price, the behaviour of this function
/// depends on which Oracle the contract has been configured with.
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
    use cosmwasm_std::testing::{
        mock_dependencies,
        mock_env,
        mock_info,
    };
    use cosmwasm_std::DepsMut;
    use pyth_sdk_terra::Price;

    use crate::msg::{
        ExecuteMsg,
        FetchPriceResponse,
    };
    use crate::state::{
        Oracle,
        State,
        STATE,
    };

    use super::query_fetch_price;

    /// set_price provides a helper that mutates our contract state. We can use this to modify the
    /// mock Oracle price in our tests.
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

    /// Quick test to make sure that when setting the price to nothing, the query no longer returns
    /// a price.
    #[test]
    pub fn test_query_fetch_price_unavailable() {
        let mut deps = mock_dependencies(&[]);

        set_price(deps.as_mut(), None);

        assert!(query_fetch_price(deps.as_ref()).is_err());
    }

    /// This test produces a stream of prices mimicing a real asset using fractional brownian
    /// motion. It uses `set_price` to feed this price stream into the contract. This can be used
    /// to test how the contract behaves in various scenarios such as price crashes.
    ///
    /// See the README for a visual graph of the price data used in this test.
    #[test]
    pub fn test_stochastic_price_action() {
        // Libraries used to generate simulated price action. (Consider switching to `noise`)
        use probability::source;
        use stochastic::gaussian::fractional::Motion;

        // We use tools from `fraction` to go from `f64` generated by stochastic to `Price`.
        use fraction::{
            Decimal,
            ToPrimitive,
        };

        // Mock Terra components.
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();
        let info = mock_info("test_contract", &[]);

        // Create a random source for our data, note that this source produces a deterministic
        // random number stream and so will always produce the same test data.
        let mut source = source::default();

        // Use a mildly jagged fractional brownian motion source (hurst 0.70) to model natural
        // price movement.
        let action = Motion::new(0.70);
        let prices = action.sample(5000, 0.1, &mut source);

        // Stochastic generates negative and positive values, here we find the smallest negative
        // value and shfit all values above this value to get a positive graph. Note that f64 does
        // not implement `Ord` so we cannot simply `prices.iter().min()` to get the minimum.
        // Instead we rely on `min_by`/`partial_cmp` and assume every `a` and `b` is comparable as
        // `a < b` for our test case.
        //
        // See `PartialOrd` vs `Ord` applied to `f64 to understand why this is important in Rust if
        // the reasoning behind this isn't clear.
        let minima = -prices
            .clone()
            .into_iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        // This is the core of our test: we iterate through the 5000 generated prices, and pass
        // them to our contract. In each iteration, we can evaluate how the contract behaves in
        // response to the price change.
        for n in 0..5000 {
            // Stochastic is generating +/- price action, shift everything above 0.0 to model more
            // realistic price data.
            let price = prices[n];
            let price = minima + price;

            // Use fraction to convert f64 to a u64 + u8, this matches how `Price` represents
            // values in Pyth as an integer + exponent.
            let price = Decimal::from(price);
            let expo = price.get_precision(); // Number of decimal places.
            let price = price * Decimal::from(10usize.pow(expo.into())); // Shift `.` to the right.
            let price = price.to_u64().unwrap(); // Get value without `.`

            // Construct our `Price` with our newly calculated `price * 10^expo` values.
            let price = Price {
                price: price as i64,
                conf:  5,
                expo:  -(expo as i32),
            };

            // Replace the price in the state before executing the contract.
            set_price(deps.as_mut(), Some(price));

            // Finally, invoke the contract itself!
            super::execute(deps.as_mut(), env, info, ExecuteMsg).unwrap();

            // Test time.
            //
            // Here, you can insert code to analyze how your contract behaves around the price
            // action above. Modifying the stochastic params and number of data points will help
            // with testing different behaviours against the Oracle. The test here will depend on
            // how you wish to use the Oracle. For example, if your program is sensitive to sudden
            // price changes, then you might want to check here for example if your program state
            // is as expected.
            println!("Replace me with a test against the resulting contract data!");
        }
    }
}
