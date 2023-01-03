use cosmwasm_std::{
    to_binary,
    Addr,
    Binary,
    Coin,
    QuerierWrapper,
    QueryRequest,
    StdResult,
    WasmQuery,
};
use schemars::JsonSchema;
use serde::{
    Deserialize,
    Serialize,
};
use std::time::Duration;

pub use pyth_sdk::{
    Price,
    PriceFeed,
    PriceIdentifier,
    ProductIdentifier,
    UnixTimestamp,
};

#[cfg(feature = "test-utils")]
pub mod test_utils;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    PriceFeed { id: PriceIdentifier },
    GetUpdateFee { vaas: Vec<Binary> },
    GetValidTimePeriod,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PriceFeedResponse {
    /// Pyth Price Feed
    pub price_feed: PriceFeed,
}

/// Queries the price on-chain
pub fn query_price_feed(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    id: PriceIdentifier,
) -> StdResult<PriceFeedResponse> {
    let price_feed_response = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.into_string(),
        msg:           to_binary(&QueryMsg::PriceFeed { id })?,
    }))?;
    Ok(price_feed_response)
}

/// Get the fee required in order to update the on-chain state with the provided
/// `price_update_vaas`.
pub fn get_update_fee(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    price_update_vaas: &[Binary],
) -> StdResult<Coin> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.into_string(),
        msg:           to_binary(&QueryMsg::GetUpdateFee {
            vaas: price_update_vaas.to_vec(),
        })?,
    }))
}

/// Get the default length of time for which a price update remains valid.
pub fn get_valid_time_period(querier: &QuerierWrapper, contract_addr: Addr) -> StdResult<Duration> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.into_string(),
        msg:           to_binary(&QueryMsg::GetValidTimePeriod)?,
    }))
}
