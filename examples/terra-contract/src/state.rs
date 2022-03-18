use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{
    Deserialize,
    Serialize,
};

use cw_storage_plus::Item;
use pyth_sdk_terra::PriceIdentifier;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub price_feed_id:      PriceIdentifier,
    pub pyth_contract_addr: Addr,
}

pub const STATE: Item<State> = Item::new("state");
