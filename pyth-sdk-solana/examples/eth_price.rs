// example usage of reading pyth price from solana/pythnet price account

use pyth_sdk_solana::load_price_feed_from_account;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};
use std::{
    thread,
    time,
};


fn main() {
    let url = "http:/pythnet.rpcpool.com";
    // Pyth ETH/USD price account on pythnet (can be found on https://pyth.network/developers/price-feed-ids#solana-mainnet-beta which has the same price feed IDs as pythnet)
    let key = "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB";
    let clnt = RpcClient::new(url.to_string());
    let eth_price_key = Pubkey::from_str(key).unwrap();

    loop {
        // get price data from key
        let mut eth_price_account = clnt.get_account(&eth_price_key).unwrap();
        let eth_price_feed =
            load_price_feed_from_account(&eth_price_key, &mut eth_price_account).unwrap();

        println!(".....ETH/USD.....");

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let maybe_price = eth_price_feed.get_price_no_older_than(current_time, 60);
        match maybe_price {
            Some(p) => {
                println!("price ........... {} x 10^{}", p.price, p.expo);
                println!("conf ............ {} x 10^{}", p.conf, p.expo);
            }
            None => {
                println!("price ........... unavailable");
                println!("conf ............ unavailable");
            }
        }


        let maybe_ema_price = eth_price_feed.get_ema_price_no_older_than(current_time, 60);
        match maybe_ema_price {
            Some(ema_price) => {
                println!(
                    "ema_price ....... {} x 10^{}",
                    ema_price.price, ema_price.expo
                );
                println!(
                    "ema_conf ........ {} x 10^{}",
                    ema_price.conf, ema_price.expo
                );
            }
            None => {
                println!("ema_price ....... unavailable");
                println!("ema_conf ........ unavailable");
            }
        }

        println!();

        thread::sleep(time::Duration::from_secs(1));
    }
}
