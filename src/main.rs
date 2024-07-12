// use reqwest::{Error, Response};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

pub type Wrapper = Vec<Hole>;

#[derive(Serialize, Deserialize, Debug)]
struct Hole {
    pub completed: bool,
    pub completed_at: String,
    pub completed_by_id: i64,
    pub completed_by_name: String,
    pub created_at: String,
    pub created_by_id: i64,
    pub created_by_name: String,
    pub expires_at: String,
    pub id: String,
    pub in_region_id: i64,
    pub in_region_name: String,
    pub in_signature: String,
    pub in_system_class: String,
    pub in_system_id: i64,
    pub in_system_name: String,
    pub max_ship_size: String,
    pub out_signature: String,
    pub out_system_id: i64,
    pub out_system_name: String,
    pub remaining_hours: i64,
    pub signature_type: String,
    pub updated_at: String,
    pub updated_by_id: i64,
    pub updated_by_name: String,
    pub wh_exits_outward: bool,
    pub wh_type: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let response = reqwest::get("https://api.eve-scout.com//v2/public/signatures?system_name=thera").await?;

    let s: Wrapper = response.json().await?;
    println!("{:<20} {:<20} {:<20} {:<20} {:<20}",
             "in_region",
             "in_system",
             "in_sig",
             "out_sig",
             "time_remaining" );
    for key in s.iter() {
        println!("{:<20} {:<20} {:<20} {:<20} {:<20}",
                 key.in_region_name,
                 key.in_system_name,
                 key.in_signature,
                 key.out_signature,
                 key.remaining_hours);
    }
    // dbg!(s);
    Ok(())
}

