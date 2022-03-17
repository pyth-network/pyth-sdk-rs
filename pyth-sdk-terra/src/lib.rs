use cosmwasm_std::{
    to_binary,
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
    PriceStatus,
    PriceIdentifier,
    ProductIdentifier,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    PriceFeed { price_id: PriceIdentifier },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PriceFeedResponse {
    /// Pyth Price Feed
    pub price_feed:        PriceFeed,
}

/// Queries the price on-chain
pub fn query_price_feed(
    querier: &QuerierWrapper,
    contract_addr: String,
    price_id: PriceIdentifier,
) -> StdResult<PriceFeedResponse> {
    let price_feed_response = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr,
        msg: to_binary(&QueryMsg::PriceFeed { price_id })?,
    }))?;
    Ok(price_feed_response)
}
