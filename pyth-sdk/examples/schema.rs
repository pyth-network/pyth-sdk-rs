use schemars::schema_for;
use serde_json::to_string_pretty;
use std::env::current_dir;
use std::fs::{
    create_dir_all,
    write,
};

use pyth_sdk::PriceFeed;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();

    let schema = &schema_for!(PriceFeed);
    let json = to_string_pretty(schema).unwrap();
    let path = out_dir.join(format!("{}.json", "price_feed"));
    write(&path, json + "\n").unwrap();
    println!("Updated {}", path.to_str().unwrap());
}
