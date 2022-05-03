use pyth_sdk_terra::{
    Price,
    PriceIdentifier,
};
use schemars::JsonSchema;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

/// InstantiateMsg is provided during contract initialization. In this example, we define the
/// message as an choice of Oracle implementation that the deployer of the contract can pick from
/// to instantiate with.
///
/// 1) PythOracle is simply an address of the Pyth contract to interact with.
/// 2) StubOracle provides a mock oracle showing how to unit test the contract against Pyth.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum InstantiateMsg {
    StubOracle {
        maybe_price: Option<Price>,
    },
    PythOracle {
        contract_addr: String,
        price_id:      PriceIdentifier,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    FetchPrice {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FetchPriceResponse {
    pub price: Price,
}
