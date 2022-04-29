use cosmwasm_std::{
    to_binary,
    Addr,
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

pub use pyth_sdk::{
    Price,
    PriceFeed,
    PriceIdentifier,
    PriceStatus,
    ProductIdentifier,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    PriceFeed { id: PriceIdentifier },
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
