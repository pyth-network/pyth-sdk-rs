use cosmwasm_std::{
    Addr,
    QuerierWrapper,
};
use schemars::JsonSchema;
use serde::{
    Deserialize,
    Serialize,
};

use cw_storage_plus::Item;
use pyth_sdk_terra::{
    query_price_feed,
    Price,
    PriceIdentifier,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub oracle: Oracle,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Oracle {
    Stub(Option<Price>),
    Pyth(Addr, PriceIdentifier),
}

impl Oracle {
    pub fn get_price(&self, querier: &QuerierWrapper) -> Option<Price> {
        match self {
            Self::Stub(maybe_price) => *maybe_price,
            Self::Pyth(contract_addr, price_id) => {
                let price_feed = query_price_feed(querier, contract_addr.to_string(), *price_id)
                    .ok()?
                    .price_feed;
                price_feed.get_ema_price()
            }
        }
    }
}

pub const STATE: Item<State> = Item::new("state");
