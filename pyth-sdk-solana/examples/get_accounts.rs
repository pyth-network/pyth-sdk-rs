// example usage of pyth-client account structure
// bootstrap all product and pricing accounts from root mapping account

use pyth_client::{
  PriceType,
  PriceStatus,
  CorpAction,
  load_mapping,
  load_product,
  load_price
};
use solana_client::{
  rpc_client::RpcClient
};
use solana_program::{
  pubkey::Pubkey
};
use std::{
  str::FromStr
};

fn get_price_type( ptype: &PriceType ) -> &'static str
{
  match ptype {
    PriceType::Unknown    => "unknown",
    PriceType::Price      => "price",
  }
}

fn get_status( st: &PriceStatus ) -> &'static str
{
  match st {
    PriceStatus::Unknown => "unknown",
    PriceStatus::Trading => "trading",
    PriceStatus::Halted  => "halted",
    PriceStatus::Auction => "auction",
  }
}

fn get_corp_act( cact: &CorpAction ) -> &'static str
{
  match cact {
    CorpAction::NoCorpAct => "nocorpact",
  }
}

fn main() {
  // get pyth mapping account
  let url = "http://api.devnet.solana.com";
  let key = "BmA9Z6FjioHJPpjT39QazZyhDRUdZy2ezwx4GiDdE2u2";
  let clnt = RpcClient::new( url.to_string() );
  let mut akey = Pubkey::from_str( key ).unwrap();

  loop {
    // get Mapping account from key
    let map_data = clnt.get_account_data( &akey ).unwrap();
    let map_acct = load_mapping( &map_data ).unwrap();

    // iget and print each Product in Mapping directory
    let mut i = 0;
    for prod_akey in &map_acct.products {
      let prod_pkey = Pubkey::new( &prod_akey.val );
      let prod_data = clnt.get_account_data( &prod_pkey ).unwrap();
      let prod_acct = load_product( &prod_data ).unwrap();

      // print key and reference data for this Product
      println!( "product_account .. {:?}", prod_pkey );
      for (key, val) in prod_acct.iter() {
        if key.len() > 0 {
          println!( "  {:.<16} {}", key, val );
        }
      }

      // print all Prices that correspond to this Product
      if prod_acct.px_acc.is_valid() {
        let mut px_pkey = Pubkey::new( &prod_acct.px_acc.val );
        loop {
          let pd = clnt.get_account_data( &px_pkey ).unwrap();
          let pa = load_price( &pd ).unwrap();

          println!( "  price_account .. {:?}", px_pkey );

          let maybe_price = pa.get_current_price();
          match maybe_price {
            Some(p) => {
              println!("    price ........ {} x 10^{}", p.price, p.expo);
              println!("    conf ......... {} x 10^{}", p.conf, p.expo);
            }
            None => {
              println!("    price ........ unavailable");
              println!("    conf ......... unavailable");
            }
          }

          println!( "    price_type ... {}", get_price_type(&pa.ptype));
          println!( "    exponent ..... {}", pa.expo );
          println!( "    status ....... {}", get_status(&pa.get_current_price_status()));
          println!( "    corp_act ..... {}", get_corp_act(&pa.agg.corp_act));

          println!( "    num_qt ....... {}", pa.num_qt );
          println!( "    valid_slot ... {}", pa.valid_slot );
          println!( "    publish_slot . {}", pa.agg.pub_slot );

          let maybe_twap = pa.get_twap();
          match maybe_twap {
            Some(twap) => {
              println!( "    twap ......... {} x 10^{}", twap.price, twap.expo );
              println!( "    twac ......... {} x 10^{}", twap.conf, twap.expo );
            }
            None => {
              println!( "    twap ......... unavailable");
              println!( "    twac ......... unavailable");
            }
          }

          // go to next price account in list
          if pa.next.is_valid() {
            px_pkey = Pubkey::new( &pa.next.val );
          } else {
            break;
          }
        }
      }
      // go to next product
      i += 1;
      if i == map_acct.num {
        break;
      }
    }

    // go to next Mapping account in list
    if !map_acct.next.is_valid() {
      break;
    }
    akey = Pubkey::new( &map_acct.next.val );
  }
}

