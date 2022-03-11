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
use std::time::Duration;

pub use pyth_sdk::{
    Price,
    PriceConf,
    PriceStatus,
    ProductIdentifier,
};

#[cfg(feature = "mainnet")]
pub const CONTRACT_ADDR: &'static str = "not-available-yet";

#[cfg(all(feature = "testnet", not(feature = "mainnet")))]
pub const CONTRACT_ADDR: &'static str = "terra1wjkzgcrg3a2jh2cyc5lekvtjydf600splmvdk4";

/// Maximum acceptable time period before price is considered to be stale.
pub const VALID_TIME_PERIOD: Duration = Duration::from_secs(60);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// price_id is currently the price public_key in Solana. It is available in https://pyth.network/markets/
    PriceInfo { price_id: Binary },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PriceInfoResponse {
    /// Pyth Price
    pub price:        Price,
    /// Price arrival time in Terra
    pub arrival_time: Timestamp,
}

/// Queries the price on-chain
pub fn query_price_info(
    querier: &QuerierWrapper,
    contract_addr: String,
    price_id: Binary,
) -> StdResult<PriceInfoResponse> {
    let terra_price_info = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr,
        msg: to_binary(&QueryMsg::PriceInfo { price_id })?,
    }))?;
    Ok(terra_price_info)
}
