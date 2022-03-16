// example usage of pyth solana account structure
// bootstrap all product and pricing accounts from root mapping account
// It is adviced to use Price directly wherever possible as described in eth_price example.
// Please use account structure only if you need it.

use pyth_sdk_solana::state::{
    load_mapping_account,
    load_price_account,
    load_product_account,
    CorpAction,
    PriceStatus,
    PriceType,
};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

fn get_price_type(ptype: &PriceType) -> &'static str {
    match ptype {
        PriceType::Unknown => "unknown",
        PriceType::Price => "price",
    }
}

fn get_status(st: &PriceStatus) -> &'static str {
    match st {
        PriceStatus::Unknown => "unknown",
        PriceStatus::Trading => "trading",
        PriceStatus::Halted => "halted",
        PriceStatus::Auction => "auction",
    }
}

fn get_corp_act(cact: &CorpAction) -> &'static str {
    match cact {
        CorpAction::NoCorpAct => "nocorpact",
    }
}

fn main() {
    // get pyth mapping account
    let url = "http://api.devnet.solana.com";
    let key = "BmA9Z6FjioHJPpjT39QazZyhDRUdZy2ezwx4GiDdE2u2";
    let clnt = RpcClient::new(url.to_string());
    let mut akey = Pubkey::from_str(key).unwrap();

    loop {
        // get Mapping account from key
        let map_data = clnt.get_account_data(&akey).unwrap();
        let map_acct = load_mapping_account(&map_data).unwrap();

        // iget and print each Product in Mapping directory
        let mut i = 0;
        for prod_akey in &map_acct.products {
            let prod_pkey = Pubkey::new(&prod_akey.val);
            let prod_data = clnt.get_account_data(&prod_pkey).unwrap();
            let prod_acct = load_product_account(&prod_data).unwrap();

            // print key and reference data for this Product
            println!("product_account .. {:?}", prod_pkey);
            for (key, val) in prod_acct.iter() {
                if key.len() > 0 {
                    println!("  {:.<16} {}", key, val);
                }
            }

            // print all Prices that correspond to this Product
            if prod_acct.px_acc.is_valid() {
                let mut px_pkey = Pubkey::new(&prod_acct.px_acc.val);
                loop {
                    let price_data = clnt.get_account_data(&px_pkey).unwrap();
                    let price_account = load_price_account(&price_data).unwrap();
                    let price = price_account.to_price(&px_pkey);

                    println!("  price_account .. {:?}", px_pkey);

                    let maybe_price = price.get_current_price();
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

                    println!(
                        "    price_type ... {}",
                        get_price_type(&price_account.ptype)
                    );
                    println!("    exponent ..... {}", price.expo);
                    println!("    status ....... {}", get_status(&price.status));
                    println!(
                        "    corp_act ..... {}",
                        get_corp_act(&price_account.agg.corp_act)
                    );

                    println!("    num_qt ....... {}", price_account.num_qt);
                    println!("    valid_slot ... {}", price_account.valid_slot);
                    println!("    publish_slot . {}", price_account.agg.pub_slot);

                    let maybe_ema_price = price.get_ema_price();
                    match maybe_ema_price {
                        Some(ema_price) => {
                            println!(
                                "    ema_price .... {} x 10^{}",
                                ema_price.price, ema_price.expo
                            );
                            println!(
                                "    ema_conf ..... {} x 10^{}",
                                ema_price.conf, ema_price.expo
                            );
                        }
                        None => {
                            println!("    ema_price .... unavailable");
                            println!("    ema_conf ..... unavailable");
                        }
                    }

                    // go to next price account in list
                    if price_account.next.is_valid() {
                        px_pkey = Pubkey::new(&price_account.next.val);
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
        akey = Pubkey::new(&map_acct.next.val);
    }
}
