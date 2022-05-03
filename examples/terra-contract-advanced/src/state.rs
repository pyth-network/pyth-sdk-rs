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

/// When the contract is initialized, the user can choose an Oracle to instantiate the contract
/// with. This choice is stored in the contract state using the `Oracle` enum. When the contract is
/// later interacted with, the selected oracle implementation is read from state in order to allow
/// the contract to dynamically choose the oracle it will use.
///
/// In this example, we do this explicitly to show how to provide a Stub oracle to test against.
/// See `contract.rs` tests for examples.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub oracle: Oracle,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Oracle {
    /// Use Pyth as an Oracle, specifying the Pyth contract address.
    Pyth(Addr, PriceIdentifier),

    /// A Stub oracle, which returns a constant price stored in contract state. This is useful for
    /// testing as it has no cross-contract interactions.
    Stub(Option<Price>),
}

impl Oracle {
    /// The `get_price` method will attempt to find the price of an asset. This method chooses the
    /// oracle it will query based on the contract state. This function is an example of how to
    /// mock oracle behaviour: note the `stub` match arm.
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
