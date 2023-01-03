use std::time::Duration;
use std::u128;
use cosmwasm_std::Coin;
use {
  cosmwasm_std::{
    from_binary,
    to_binary,
    Binary,
    ContractResult,
    QuerierResult,
    SystemError,
    SystemResult,
  },
  std::collections::HashMap,
};
use pyth_sdk::{Price, PriceFeed, PriceIdentifier};
use crate::{PriceFeedResponse, QueryMsg};

/// Mock version of Pyth for testing cosmwasm contracts.
/// This mock stores some price feeds and responds to query messages.
#[derive(Clone)]
pub struct MockPyth {
  pub valid_time_period: Duration,
  pub fee_per_vaa: Coin,
  pub feeds: HashMap<PriceIdentifier, PriceFeed>,
}

impl MockPyth {
  pub fn new(valid_time_period: Duration,
             fee_per_vaa: Coin,
             feeds: &[PriceFeed]) -> Self {
    let mut feeds_map = HashMap::new();
    for feed in feeds {
      feeds_map.insert(feed.id, *feed);
    }

    MockPyth { valid_time_period, fee_per_vaa, feeds: feeds_map }
  }

  /// Add a price feed that will be returned on queries.
  pub fn add_feed(&mut self, feed: PriceFeed) {
    self.feeds.insert(feed.id, feed);
  }

  /// Add a price feed containing `price` as both the current price and EMA.
  pub fn add_feed_with_price(&mut self, id: PriceIdentifier, price: Price) {
    let feed = PriceFeed::new(
      id,
      price,
      price,
    );
    self.feeds.insert(id, feed);
  }

  /// Handler for processing query messages. See the tests in `contract.rs` for how to use this
  /// handler within your tests.
  pub fn handle_wasm_query(&self, msg: &Binary) -> QuerierResult {
    let query_msg = from_binary::<QueryMsg>(msg);
    match query_msg {
      Ok(QueryMsg::PriceFeed { id }) => match self.feeds.get(&id) {
        Some(feed) => SystemResult::Ok(ContractResult::Ok(
          to_binary(&PriceFeedResponse {
            price_feed: *feed,
          })
            .unwrap(),
        )),
        None => SystemResult::Ok(ContractResult::Err("unknown price feed".into())),
      },
      Ok(QueryMsg::GetValidTimePeriod) => SystemResult::Ok(ContractResult::Ok(to_binary(&self.valid_time_period).unwrap())),
      Ok(QueryMsg::GetUpdateFee { vaas}) => {
        let new_amount = self.fee_per_vaa.amount.u128().checked_mul(vaas.len() as u128).unwrap();
        SystemResult::Ok(ContractResult::Ok(to_binary(&Coin::new(new_amount, &self.fee_per_vaa.denom)).unwrap()))
      },
      Err(_e) => SystemResult::Err(SystemError::InvalidRequest {
        error:   "Invalid message".into(),
        request: msg.clone(),
      }),
    }
  }
}
