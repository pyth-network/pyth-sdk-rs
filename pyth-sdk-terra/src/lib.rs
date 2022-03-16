use cosmwasm_std::{
    to_binary,
    Binary,
    QuerierWrapper,
    QueryRequest,
    StdResult,
    Timestamp,
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

#[cfg(feature = "mainnet")]
pub const CONTRACT_ADDR: &'static str = "not-available-yet";

#[cfg(all(feature = "testnet", not(feature = "mainnet")))]
pub const CONTRACT_ADDR: &'static str = "terra1wjkzgcrg3a2jh2cyc5lekvtjydf600splmvdk4";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// price_id is currently the price public_key in Solana. It is available in https://pyth.network/markets/
    PriceInfo { price_id: PriceIdentifier },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PriceInfoResponse {
    /// Pyth Price Feed
    pub price_feed:        PriceFeed,
    /// The timestamp that the price was published to the wormhole
    pub time: Timestamp,
}

/// Queries the price on-chain
pub fn query_price_info(
    querier: &QuerierWrapper,
    contract_addr: String,
    price_id: PriceIdentifier,
) -> StdResult<PriceInfoResponse> {
    let terra_price_info = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr,
        msg: to_binary(&QueryMsg::PriceInfo { price_id })?,
    }))?;
    Ok(terra_price_info)
}
