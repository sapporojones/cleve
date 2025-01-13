use std::fmt::Error as FError;
use std::io::{Read, Write};
use std::io;
use std::io::Cursor;
use futures_util::StreamExt;
use std::io::Error as IError;
use std::ptr::null;
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};
use chrono::Utc;
use serde_json::{json, to_string, Value};
use std::time::SystemTime;
use std::string::String;
use std::thread::current;
use std::path::{Path, PathBuf};
use thiserror::Error;
use miette::{Diagnostic, Result};
use reqwest::{
    Client,
    header::USER_AGENT,
    Error
};
use bzip2_rs::DecoderReader;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};

use zip::ZipArchive;
use zip_extensions::read::ZipArchiveExtensions;
use serde_json::Value::Null;
use rusqlite;
// use error_chain::error_chain;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use zip_extensions::zip_extract;
use log::debug;

#[derive(Error, Diagnostic, Debug)]
enum MyError {
    #[error("IO Error")]
    IO(#[from] std::io::Error),
    #[error("FE Error")]
    FE(#[from] std::fmt::Error),
    #[error("Reqwest Error")]
    RE(#[from] reqwest::Error),
    #[error("SQLx Error")]
    SqE(#[from] sqlx::Error),
    #[error("Other Error")]
    Custom(String),
}
// for write_all()





// static DB: Pool<Sqlite> = SqlitePool::connect(DB_URL).await.unwrap();

async fn db_connect() -> Pool<Sqlite>{
    let db_url: &str = "sqlite://sqlite-latest.sqlite";
    let pool = SqlitePool::connect(db_url).await.expect("Unable to connect to database");
    pool
}


// async fn get_db_conn(dbpool: &Pool<Sqlite>) -> Pool<Sqlite> {
//     // dbpool.get().unwrap().acquire().await.unwrap()
//     dbpool.acquire().await.expect("Unable to create new pool connection");
//
//
// }

pub type Incursions = Vec<IncursionStruct>;

#[derive(Serialize, Deserialize)]
pub struct IncursionStruct {
    pub constellation_id: i64,
    pub faction_id: i64,
    pub has_boss: bool,
    pub infested_solar_systems: Vec<i64>,
    pub influence: f64,
    pub staging_solar_system_id: i64,
    pub state: String,
    #[serde(rename = "type")]
    pub incursion_type: String,
}
#[derive(Serialize, Deserialize)]
pub struct RegionInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constellations: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub name: String,
    pub region_id: i64,
}


#[derive(Serialize, Deserialize)]
pub struct ConstInfo {
    pub constellation_id: i64,
    pub name: String,
    pub position: Position,
    pub region_id: i64,
    pub systems: Vec<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct ConstPlanet {
    pub moons: Vec<i64>,
    pub planet_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ConstPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub type SystemZkb = Vec<SystemZkbStruct>;

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemZkbStruct {
    pub killmail_id: i64,
    pub zkb: Zkb,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Zkb {
    #[serde(rename = "locationID")]
    pub location_id: i64,
    pub hash: String,
    pub fitted_value: f64,
    pub dropped_value: f64,
    pub destroyed_value: f64,
    pub total_value: f64,
    pub points: i64,
    pub npc: bool,
    pub solo: bool,
    pub awox: bool,
    pub labels: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemInfo {
    pub constellation_id: i64,
    pub name: String,
    pub planets: Vec<Planet>,
    pub position: Position,
    pub security_class: String,
    pub security_status: f64,
    pub star_id: i64,
    pub stargates: Vec<i64>,
    pub system_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Planet {
    pub planet_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asteroid_belts: Option<Vec<i64>>,
    pub moons: Option<Vec<i64>>,
}

#[derive(Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}



pub type SysJumps = Vec<Jumps>;

#[derive(Serialize, Deserialize)]
pub struct Jumps {
    pub ship_jumps: i64,
    pub system_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct EveStatus {
    pub players: i64,
    pub server_version: String,
    pub start_time: String,
}

pub type Kills = Vec<KillsStruct>;

#[derive(Serialize, Deserialize)]
pub struct KillsStruct {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npc_kills: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_kills: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_kills: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_id: Option<i64>,
}

type EveScout = Vec<Hole>;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct CharInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alliance_id: Option<u64>,
    // pub alliance_id: i64,
    pub birthday: String,
    pub bloodline_id: i64,
    pub corporation_id: i64,
    pub description: String,
    pub gender: String,
    pub name: String,
    pub race_id: i64,
    pub security_status: f64,
}

#[derive(Serialize, Deserialize)]
pub struct EveWho {
    pub info: Vec<Info>,
    pub characters: Vec<Character>,
}

#[derive(Serialize, Deserialize)]
pub struct Character {
    pub character_id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Info {
    pub alliance_id: i64,
    pub name: String,
    #[serde(rename = "memberCount")]
    pub member_count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct AllianceID {
    alliance_id: u64,
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CorpInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alliance_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ceo_id: Option<i64>,
    pub creator_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_founded: Option<String>,
    pub description: String,
    pub home_station_id: i64,
    pub member_count: i64,
    pub name: String,
    pub shares: i64,
    pub tax_rate: f64,
    pub ticker: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub war_eligible: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct AllianceInfo {
    pub creator_corporation_id: i64,
    pub creator_id: i64,
    pub date_founded: String,
    pub executor_corporation_id: i64,
    pub name: String,
    pub ticker: String,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

pub type Campaigns = Vec<CampaignStruct>;

#[derive(Serialize, Deserialize)]
pub struct CampaignStruct {
    pub attackers_score: f64,
    pub campaign_id: i64,
    pub constellation_id: i64,
    pub defender_id: i64,
    pub defender_score: f64,
    pub event_type: EventType,
    pub solar_system_id: i64,
    pub start_time: String,
    pub structure_id: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    #[serde(rename = "ihub_defense")]
    IhubDefense,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CcpKillmail {
    pub attackers: Vec<Attacker>,
    pub killmail_id: i64,
    pub killmail_time: String,
    pub solar_system_id: i64,
    pub victim: Victim,
}

// #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
// pub struct Attacker {
//     pub damage_done: i64,
//     pub final_blow: bool,
//     pub security_status: f64,
// }

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Attacker {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alliance_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_id: Option<i64>,
    pub corporation_id: i64,
    pub damage_done: i64,
    pub final_blow: bool,
    pub security_status: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_type_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weapon_type_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Victim {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alliance_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_id: Option<i64>,
    pub corporation_id: i64,
    pub damage_taken: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction_id: Option<i64>,
    pub items: Vec<Option<serde_json::Value>>,
    pub position: KillPosition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_type_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct KillPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Subcommand)]
enum Commands {
    /// For listing public travel wormhole routes from Thera or Turnur
    Travel,
    /// For Thera specific wormhole information
    Thera,
    /// For Turnur specific wormhole information
    Turnur,
    /// List information about active incursions
    Incursions,
    /// For information about a character
    Pilot {
        /// Name of character to lookup, if character name contains spaces quotation marks must be used
        character_name: String,
    },
    /// Retrieve information about a specified system
    System {
        system_name: String,
    },
    /// Retrieve current status of the Tranquility server
    Status,
    /// Returns information about current sov timers (SOON TO BE DEPRECATED)
    Timers,
    /// Update SDE components
    Update,

}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let start = SystemTime::now();
    let cli = Cli::parse();


    match &cli.command {
        Some(Commands::Travel {  }) => {
            evescout().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Pilot { character_name }) =>{
            shlookup(character_name.as_str()).await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());

        }
        Some(Commands::Thera {  }) => {
            thera().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Turnur {  }) => {
            turnur().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Status { }) => {
            status().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::System {system_name}) => {
            system_stats(system_name).await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Incursions { }) => {
            incursions().await.expect("Unable to fetch incursion data");

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Update { }) => {
            let _ = get_sde_components().await;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Timers { }) => {
            timers().await;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        None => {
            println!("No command specified.  Please supply a command or re-run with --help for help.");
        }
    }
    Ok(())
}

async fn evescout() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let thera_response = client.get("https://api.eve-scout.com//v2/public/signatures?system_name=thera").send().await?;
    let thera: EveScout = thera_response.json().await?;
    let turnur_response = client.get("https://api.eve-scout.com//v2/public/signatures?system_name=turnur").send().await?;
    let turnur: EveScout = turnur_response.json().await?;

    println!("\nThera\n{:<20} {:<15} {:<15} {:<15} {:<15}",
             "in_region",
             "in_system",
             "in_sig",
             "out_sig",
             "time_remaining" );
    for key in thera.iter() {
        println!("{:<20} {:<15} {:<15} {:<15} {:<15}",
                 key.in_region_name,
                 key.in_system_name,
                 key.in_signature,
                 key.out_signature,
                 key.remaining_hours);
    }
    println!("\n");
    println!("Turnur\n{:<20} {:<15} {:<15} {:<15} {:<15}",
             "in_region",
             "in_system",
             "in_sig",
             "out_sig",
             "time_remaining" );
    for key in turnur.iter() {
        println!("{:<20} {:<15} {:<15} {:<15} {:<15}",
                 key.in_region_name,
                 key.in_system_name,
                 key.in_signature,
                 key.out_signature,
                 key.remaining_hours);
    }
    println!("\n");
    Ok(())
}

async fn thera() -> Result<(), reqwest::Error> {
    let thera_response = reqwest::get("https://api.eve-scout.com//v2/public/signatures?system_name=thera").await?;
    let thera: EveScout = thera_response.json().await?;
    println!("\nThera\n{:<20} {:<15} {:<15} {:<15} {:<15}",
             "in_region",
             "in_system",
             "in_sig",
             "out_sig",
             "time_remaining" );
    for key in thera.iter() {
        println!("{:<20} {:<15} {:<15} {:<15} {:<15}",
                 key.in_region_name,
                 key.in_system_name,
                 key.in_signature,
                 key.out_signature,
                 key.remaining_hours);
    }
    println!("\n");
    Ok(())
}

async fn turnur() -> Result<(), reqwest::Error> {
    let turnur_response = reqwest::get("https://api.eve-scout.com//v2/public/signatures?system_name=turnur").await?;
    let turnur: EveScout = turnur_response.json().await?;
    println!("\nTurnur\n{:<20} {:<15} {:<15} {:<15} {:<15}",
             "in_region",
             "in_system",
             "in_sig",
             "out_sig",
             "time_remaining" );
    for key in turnur.iter() {
        println!("{:<20} {:<15} {:<15} {:<15} {:<15}",
                 key.in_region_name,
                 key.in_system_name,
                 key.in_signature,
                 key.out_signature,
                 key.remaining_hours);
    }
    println!("\n");

    Ok(())

}

async fn status() -> Result<(), reqwest::Error> {
    let url = "https://esi.evetech.net/latest/status/?datasource=tranquility";
    let status_response = reqwest::get(url).await?;
    let status: EveStatus = status_response.json().await?;
    println!("\nPlayers online: {}", status.players.to_string().as_str());
    println!("Current server version: {}", status.server_version.to_string().as_str());
    println!("Server start time: {}\n", status.start_time.to_string().as_str());

    Ok(())
}

async fn shlookup(char_name: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    // // known character ids for testing with:
    // // sappo = 772506501
    // // billy = 1826057122
    // // d1ck = 749151393
    // // comment out below for release version
    // let char_id: &str = "772506501";
    // uncomment below to accept char id from user via command line args
    let ci = char_name;
    let char_id = char_search(ci, client.clone()).await?;
    let p: CharInfo  = public_info(char_id.as_str(), client.clone()).await?;

    let corpid: i64 = p.corporation_id;
    let c: CorpInfo = corp_info(corpid.to_string().as_str(), client.clone()).await?;


    let aid = c.alliance_id.clone();

    let zs: Value = get_zkb_stats(char_id.clone(), client.clone()).await?;

    let mut kills_vec = Vec::new();
    let mut losses_vec = Vec::new();
    let mut ships_kills_vec = Vec::new();
    let mut ships_loss_vec = Vec::new();

    // knobs





    let kills_url = format!("https://zkillboard.com/api/kills/characterID/{}/", char_id);

    let kills_response = client.get(kills_url).send().await?;
    let zkb: Value = kills_response.json().await?;

    let kill_parse_limit: usize = if zs["shipsDestroyed"].as_i64().expect("Couldn't determine number of ships interacted with") < i64::from(5) {
        let parse_limit = zs["shipsDestroyed"].as_u64().expect("Couldn't determine number of ships interacted with");
        parse_limit as usize
    } else {
        let parse_limit = 5;
        parse_limit
    };
    let loss_parse_limit: usize = if zs["shipsLost"].as_i64().expect("Couldn't determine number of ships interacted with") < i64::from(5) {
        let parse_limit = zs["shipsLost"].as_u64().expect("Couldn't determine number of ships interacted with");
        parse_limit as usize
    } else {
        let parse_limit = 5;
        parse_limit
    };


    // let parse_limit = 5;
    let mut current_kill: usize = 0;

    while current_kill < kill_parse_limit {
        let mr_id: String = zkb[current_kill]["killmail_id"].to_string();
        let mr_hash: String = zkb[current_kill]["zkb"]["hash"].to_string().replace("\"", "");

        let mr_kill: CcpKillmail = kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;
        kills_vec.push(mr_kill);
        current_kill+=1;
    }
    let mut attack_ship = Vec::new();
    for kill in kills_vec.iter() {
        let killed_with_j: String = item_lookup(
            kill.victim.ship_type_id.expect("Can't find victim ship type")
                .to_string()
                .replace("\"", ""),
            client.clone(),
        ).await?;
        let killed_with: String = killed_with_j.replace("\"", "");
        ships_kills_vec.push(killed_with);

        for attacker in kill.attackers.iter() {

            if attacker.character_id.is_none() {
                // println!("attacker.character_id is none");
                let attack_ship_id = attacker.ship_type_id.expect("Could not determine ship ID").to_string();
                let attack_tmp = item_lookup(attack_ship_id, client.clone()).await?;
                attack_ship.push(attack_tmp)
            } else if attacker.character_id.expect("Can't find character ID on kill for some reaosn").to_string() == char_id {
                if attacker.ship_type_id.is_none() {
                    // println!("attacker.character_id = char_id");
                    let attack_ship_id = attacker.weapon_type_id.expect("Can't find weapon type used").to_string();
                    let attack_tmp = item_lookup(attack_ship_id, client.clone()).await?;
                    attack_ship.push(attack_tmp)
                } else {
                    // println!("attacker.character_id else condition");
                    let attack_ship_id = attacker.ship_type_id.expect("Could not determine ship ID").to_string();
                    let attack_tmp = item_lookup(attack_ship_id, client.clone()).await?;
                    attack_ship.push(attack_tmp)
                }

            }
        }


    }

    let loss_url = format!("https://zkillboard.com/api/losses/characterID/{}/", char_id);

    let loss_response = client.get(loss_url).send().await?;
    let zkb: Value = loss_response.json().await?;

    let mut current_loss = 0;
    while current_loss < loss_parse_limit {
        let mr_id: String = zkb[current_loss]["killmail_id"].to_string();
        let mr_hash: String = zkb[current_loss]["zkb"]["hash"].to_string().replace("\"", "");

        let mr_kill: CcpKillmail = kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;
        losses_vec.push(mr_kill);
        current_loss+=1;
    }
    for loss in losses_vec.iter() {
        let killed_with_j: String = item_lookup(
            loss.victim.ship_type_id.expect("Can't find victim ship type")
                .to_string()
                .replace("\"", ""),
            client.clone(),
        ).await?;
        let lost_with: String = killed_with_j.replace("\"", "");
        ships_loss_vec.push(lost_with);

    }



    // let mr_kill: CcpKillmail = get_mr_kill_info(char_id.clone().to_string(), client.clone()).await?;
    // let mr_loss: CcpKillmail = get_mr_loss_info(char_id.clone().to_string(), client.clone()).await?;








    println!("\n \nBasic info:");
    println!("Name: {}", p.name.to_string().replace("\"", ""));
    let bday_raw: String = p.birthday.to_string().replace("\"", "");
    let bday_clean: String = date_parse(&bday_raw);

    println!("Birthday: {}", bday_clean);


    let sec_status: String = p.security_status.to_string();
    println!("Security Status: {:}", &sec_status.as_str().replace("\"", ""));

    println!(
        "\nCorporation: {} [{}]",
        c.name.to_string().replace("\"", ""),
        c.ticker.to_string().replace("\"", "")
    );
    // println!("Ticker: {}", c["ticker"]);
    println!("Corporation members: {}", c.member_count);
    println!("Corporation tax rate: {}", c.tax_rate);

    let corp_bday_raw: String = c.date_founded.unwrap().to_string().replace("\"", "");
    let corp_bday: String = date_parse(&corp_bday_raw);
    println!("Corporation founded on: {}", corp_bday);
    println!(
        "Corporation evewho: https://evewho.com/corporation/{}",
        p.corporation_id
    );

    match c.alliance_id {
        None => {
            println!("\nAlliance:  Corporation is not a member of an alliance.")
        },
        Some(_aid) => {
            let alliance_info: AllianceInfo = alliance_info(aid.unwrap().to_string(), client.clone()).await?;
            println!(
                "\nAlliance: {} [{}]",
                alliance_info.name.to_string().replace("\"", ""),
                alliance_info.ticker.to_string().replace("\"", "")
            );

            let alliance_bday_raw: String = alliance_info.date_founded.to_string().replace("\"", "");
            let alliance_bday: String = date_parse(&alliance_bday_raw);

            println!("Alliance founded on: {}", alliance_bday);
            println!(
                "Alliance evewho: https://evewho.com/alliance/{:?}",
                c.alliance_id.unwrap()
            )

        }

    };

    println!("\nZKB Stats:");
    println!(
        "Character Zkb: https://zkillboard.com/character/{}/",
        char_id
    );
    println!("\nAlltime kills: {}", zs["shipsDestroyed"]);
    println!("Alltime losses: {}", zs["shipsLost"]);
    println!("Solo kills: {}", zs["soloKills"]);
    println!("Solo losses: {}", zs["soloLosses"]);


    println!("\nMost recent kills:");
    let mut idx = 0;
    for kill in kills_vec.iter() {
        let killtime = kill.killmail_time.to_string();
        let killtime_clean: String = date_parse(&killtime);
        let kill_diff= date_calc(killtime.clone()).await?;
        let killed_with = &ships_kills_vec[idx];
        let kill_system = get_timer_solar_name(kill.solar_system_id.to_string(), client.clone()).await?;
        let kill_const = get_timer_const_id(kill.solar_system_id.to_string(), client.clone()).await?;
        let kill_region = get_timer_region_id(kill_const.to_string(), client.clone()).await?;
        let kill_region_name = get_timer_region_name(kill_region.to_string(), client.clone()).await?;
        println!(
            "{} days ago on {} killed a(n) {} while flying a {} in {} - {}",
            kill_diff, &killtime_clean, killed_with, attack_ship[idx], kill_system, kill_region_name
        );

        idx+=1;
    }

    println!("\nMost recent Losses:");
    let mut idx = 0;
    for loss in losses_vec.iter() {
        let losstime = loss.killmail_time.to_string();
        let losstime_clean: String = date_parse(&losstime);
        let loss_diff = date_calc(losstime.clone()).await?;
        let lost_ship = &ships_loss_vec[idx];
        let loss_system = get_timer_solar_name(loss.solar_system_id.to_string(), client.clone()).await?;
        let loss_const = get_timer_const_id(loss.solar_system_id.to_string(), client.clone()).await?;
        let loss_region = get_timer_region_id(loss_const.to_string(), client.clone()).await?;
        let loss_region_name = get_timer_region_name(loss_region.to_string(), client.clone()).await?;
        println!(
            "{} days ago on {} lost a(n) {} in {} - {}",
            loss_diff, &losstime_clean, lost_ship, loss_system, loss_region_name
        );

        idx+=1;
    }



    println!("\n \n");
    Ok(())

}

async fn char_search(char_name: &str, client: Client) -> Result<String, reqwest::Error> {
    let payloadstring = format!("[{:?}]", char_name);
    // println!("Searching for {:?}...", char_name);

    let url = "https://esi.evetech.net/latest/universe/ids/?datasource=tranquility&language=en";

    let resp = client.post(url)
        .body(payloadstring)
        .send()
        .await?;
    let lookup: Value = resp.json().await?;
    let char_id = lookup["characters"][0]["id"].to_string();

    println!("{:?} found as {:?}...", char_name, char_id);
    Ok(char_id)
}

async fn public_info(char_id: &str, client: Client) -> Result<CharInfo, reqwest::Error> {
    // println!("Fetching public info...");
    let url: String = format!(
        "https://esi.evetech.net/latest/characters/{char_id}/?datasource=tranquility"
    );

    let publicinfo_response = client.get(&url).send().await?;
    let p: CharInfo = publicinfo_response.json().await?;

    Ok(p)
}

async fn corp_info(corporation_id: &str, client: Client) -> Result<CorpInfo, reqwest::Error> {
    // println!("Fetching corporation info...");
    let url: String = format!(
        "https://esi.evetech.net/latest/corporations/{}/?datasource=tranquility",
        corporation_id
    );

    let corp_response = client.get(url).send().await?;
    let corp_info: CorpInfo = corp_response.json().await?;
    Ok(corp_info)
}

async fn alliance_info(corporation_id: String, client: Client) -> Result<AllianceInfo, reqwest::Error> {
    // println!("Fetching alliance information...");
    let url: String = format!(
        "https://esi.evetech.net/latest/alliances/{}/?datasource=tranquility",
        corporation_id
    );
    let alliance_response = client.get(url).send().await?;
    let alliance_info: AllianceInfo = alliance_response.json().await?;


    Ok(alliance_info)
}

async fn get_mr_kill_info(char_id: String, client: Client) -> Result<CcpKillmail, reqwest::Error> {
    // println!("Fetching most recent kill data...");
    let url = format!("https://zkillboard.com/api/kills/characterID/{}/", char_id);

    let kills_response = client.get(url).send().await?;
    let zkb: Value = kills_response.json().await?;

    let mr_id: String = zkb[0]["killmail_id"].to_string();
    let mr_hash: String = zkb[0]["zkb"]["hash"].to_string().replace("\"", "");

    let mr_kill: CcpKillmail = kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;

    Ok(mr_kill)
}

async fn get_kill_info(char_id: String, client: Client) -> Result<CcpKillmail, reqwest::Error> {
    // println!("Fetching most recent kill data...");
    let url = format!("https://zkillboard.com/api/kills/characterID/{}/", char_id);

    let kills_response = client.get(url).send().await?;
    let zkb: Value = kills_response.json().await?;

    let mr_id: String = zkb[0]["killmail_id"].to_string();
    let mr_hash: String = zkb[0]["zkb"]["hash"].to_string().replace("\"", "");

    let mr_kill: CcpKillmail = kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;

    Ok(mr_kill)
}

async fn get_loss_info(char_id: String, client: Client) -> Result<CcpKillmail, reqwest::Error> {
    // println!("Fetching most recent kill data...");
    let url = format!("https://zkillboard.com/api/losses/characterID/{}/", char_id);

    let kills_response = client.get(url).send().await?;
    let zkb: Value = kills_response.json().await?;

    let mr_id: String = zkb[0]["killmail_id"].to_string();
    let mr_hash: String = zkb[0]["zkb"]["hash"].to_string().replace("\"", "");

    let mr_kill: CcpKillmail = kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;

    Ok(mr_kill)
}

async fn get_mr_loss_info(char_id: String, client: Client) -> Result<CcpKillmail, reqwest::Error> {
    // println!("Fetching most recent loss data...");
    let url = format!("https://zkillboard.com/api/losses/characterID/{}/", char_id);

    let losses = client.get(url).send().await?;
    let zkb: Value = losses.json().await?;

    let mr_id: String = zkb[0]["killmail_id"].to_string();
    let mr_hash: String = zkb[0]["zkb"]["hash"].to_string().replace("\"", "");

    let mr_loss: CcpKillmail = kill_resolve(mr_id.to_string(), mr_hash.to_string(),client.clone() ).await?;

    Ok(mr_loss)
}

async fn get_zkb_stats(char_id: String, client: Client) -> Result<Value, reqwest::Error> {
    println!("Fetching zkill stats data...");
    let url = format!("https://zkillboard.com/api/stats/characterID/{}/", char_id);

    let response = client.get(url).send().await?;
    let zkb = response.json().await?;

    Ok(zkb)
}

async fn kill_resolve(kill_id: String, kill_hash: String, client: Client) -> Result<CcpKillmail, reqwest::Error> {
    let url = format!(
        "https://esi.evetech.net/latest/killmails/{}/{}/?datasource=tranquility",
        kill_id, kill_hash
    );

    let response = client.get(url).send().await?;

    let kill_info: CcpKillmail = response.json().await?;

    Ok(kill_info)
}

async fn date_calc(date_string: String) -> Result<i64, reqwest::Error> {
    let dt: Vec<&str> = date_string.split("T").collect();
    let date = dt[0].replace("\"", "");
    let today = Utc::now();
    let todate = today.date_naive();

    let pfs = chrono::NaiveDate::parse_from_str;

    let naive_dt = pfs(&date, "%Y-%m-%d").expect("unable to parse kill date");

    let diff = todate.signed_duration_since(naive_dt);
    let days = diff.num_days();
    // let years = days / 365;
    // let remaining_days = days % 365;
    // let months = remaining_days / 30;
    // let rem_days = remaining_days % 30;

    Ok(days)
}

fn date_parse(date_string: &String) -> String {
    let dt: Vec<&str> = date_string.split("T").collect();
    let date = dt[0].replace("\"", "");

    let pfs = chrono::NaiveDate::parse_from_str;

    let naive_dt = pfs(&date, "%Y-%m-%d").expect("unable to parse kill date");
    naive_dt.to_string()
}

// async fn item_lookup(item_id: String, client: Client, dbconnect: &Pool<SqlitePool>) -> Result<Value, reqwest::Error> {
async fn item_lookup(item_id: String, client: Client) -> Result<String, reqwest::Error> {
    // let ps = format!("[{}]", item_id);
    // let payload = json!(ps);
    // let pl = payload.as_str().unwrap();
    //
    // let url = "https://esi.evetech.net/latest/universe/names/?datasource=tranquility&language=en";
    //
    // let response = client.post(url)
    //     .body(ps)
    //     .send()
    //     .await?;
    //
    // let res = response.json().await?;

    // let dbpool = get_db_conn(dbconnect);
    let db_connect = db_connect().await;
    let pool = db_connect.acquire().await.expect("Unable to create new pool connection");

    let item = sqlx::query!("SELECT  typeName FROM invTypes WHERE typeID IS ?", item_id)
        .fetch_one(&db_connect)
        .await
        .expect("Unable to query the database");



    Ok(item.typeName.expect("Unable to return database record"))
}
async fn legacy_item_lookup(item_id: String, client: Client) -> Result<Value, reqwest::Error> {
    let ps = format!("[{}]", item_id);
    let payload = json!(ps);
    let pl = payload.as_str().unwrap();

    let url = "https://esi.evetech.net/latest/universe/names/?datasource=tranquility&language=en";

    let response = client.post(url)
        .body(ps)
        .send()
        .await?;

    let res: Value = response.json().await?;



    Ok(res)
}

// async fn name_lookup(item_name: String, client: Client, dbconnect: &Pool<SqlitePool>) -> Result<Value, reqwest::Error> {
async fn name_lookup(item_name: String, client: Client) -> Result<Value, reqwest::Error> {
    let ps = format!("[\"{item_name}\"]");

    let url = "https://esi.evetech.net/latest/universe/ids/?datasource=tranquility&language=en";

    let response = client.post(url)
        .body(ps)
        .send()
        .await?;

    let lookup: Value = response.json().await?;

    Ok(lookup)
}

async fn get_jumps(system_id: &str, client: Client) -> Result<String, reqwest::Error> {
    let url = "https://esi.evetech.net/latest/universe/system_jumps/?datasource=tranquility";
    let sysjumps = client.get(url).send().await?;
    let jumpstruct: SysJumps = sysjumps.json().await?;

    let mut j: i64 = 0;
    for key in jumpstruct.iter() {
        if key.system_id.to_string().as_str() == system_id {
            j = key.ship_jumps;

        };
    };
    let jumps: String = j.to_string();
    Ok(jumps.to_string())
}

async fn get_gates(system_id: &str, client: Client) -> Result<String, reqwest::Error> {
    let url = format!("https://esi.evetech.net/latest/universe/systems/{system_id}/");
    let gate_response = client.get(url).send().await?;
    let gates: SystemInfo = gate_response.json().await?;
    let num_gates = gates.stargates.len().to_string();
    Ok(num_gates)
}

async fn get_num_kills(system_id: &str, client: Client) -> Result<Vec<String>, reqwest::Error> {
    let url = "https://esi.evetech.net/latest/universe/system_kills/?datasource=tranquility";
    let kills_response = client.get(url).send().await?;
    let killsj: Value = kills_response.json().await?;

    let mut kills_vec: Vec<String> = Vec::new();
    for key in killsj.as_object().iter() {
        if key["system_id"].to_string().as_str() == system_id {

            kills_vec.push(key["npc_kills"].to_string());
            kills_vec.push(key["pod_kills"].to_string());
            kills_vec.push(key["ship_kills"].to_string());

        };
    };

    Ok(kills_vec)
}

async fn get_npc_kills(system_id: &str, client: Client) -> Result<String, reqwest::Error> {
    let url = "https://esi.evetech.net/latest/universe/system_kills/?datasource=tranquility";
    let kills_response = client.get(url).send().await?;
    let kills_json: Value = kills_response.json().await?;
    let mut k: i64 = 0;
    for key in kills_json.as_object().iter() {
        if key["system_id"].to_string().as_str() == system_id {
            k = key["npc_kills"].as_i64().unwrap();
        };
    };
    let kills: String = k.to_string();
    Ok(kills)
}

async fn get_system_kills(system_id: &str, client: Client) -> Result<SystemZkb, reqwest::Error> {
    let url = format!("https://zkillboard.com/api/solarSystemID/{system_id}/");
    let zkbsys_response = client.get(url).send().await?;

    let zkbsysj: SystemZkb = zkbsys_response.json().await?;
    Ok(zkbsysj)

}

async fn get_solar_name(system_id: String, client: Client) -> Result<String, reqwest::Error> {

    let db_connect = db_connect().await;
    let pool = db_connect.acquire().await.expect("Unable to create new pool connection");

    let system = sqlx::query!("SELECT solarSystemName FROM mapSolarSystems WHERE solarSystemID IS ?", system_id)
        .fetch_one(&db_connect)
        .await
        .expect("Unable to query the database");



    Ok(system.solarSystemName.expect("Unable to return database record"))
}

async fn get_timer_solar_id(system_id: String, client: Client) -> Result<i64, reqwest::Error> {

    let db_connect = db_connect().await;
    let pool = db_connect.acquire().await.expect("Unable to create new pool connection");

    let system = sqlx::query!("SELECT solarSystemID FROM mapSolarSystems WHERE solarSystemName IS ?", system_id)
        .fetch_one(&db_connect)
        .await
        .expect("Unable to query the database");



    Ok(system.solarSystemID)
}

async fn get_timer_solar_name(system_id: String, client: Client) -> Result<String, reqwest::Error> {

    let db_connect = db_connect().await;
    let pool = db_connect.acquire().await.expect("Unable to create new pool connection");

    let system = sqlx::query!("SELECT solarSystemName FROM mapSolarSystems WHERE solarSystemID IS ?", system_id)
        .fetch_one(&db_connect)
        .await
        .expect("Unable to query the database");



    Ok(system.solarSystemName.expect("Unable to return database record"))
}
async fn get_timer_const_id(system_id: String, client: Client) -> Result<i64, reqwest::Error> {

    let db_connect = db_connect().await;
    let pool = db_connect.acquire().await.expect("Unable to create new pool connection");

    let system = sqlx::query!("SELECT constellationID FROM mapSolarSystems WHERE solarSystemID IS ?", system_id)
        .fetch_one(&db_connect)
        .await
        .expect("Unable to query the database");



    Ok(system.constellationID.expect("Unable to return database record"))
}

async fn get_timer_region_id(const_id: String, client: Client) -> Result<i64, reqwest::Error> {

    let db_connect = db_connect().await;
    let pool = db_connect.acquire().await.expect("Unable to create new pool connection");

    let system = sqlx::query!("SELECT regionID FROM mapConstellations WHERE constellationID IS ?", const_id)
        .fetch_one(&db_connect)
        .await
        .expect("Unable to query the database");



    Ok(system.regionID.expect("Unable to return database record"))
}

async fn get_timer_region_name(region_id: String, client: Client) -> Result<String, reqwest::Error> {

    let db_connect = db_connect().await;
    let pool = db_connect.acquire().await.expect("Unable to create new pool connection");

    let system = sqlx::query!("SELECT regionName FROM mapRegions WHERE regionID IS ?", region_id)
        .fetch_one(&db_connect)
        .await
        .expect("Unable to query the database");



    Ok(system.regionName.expect("Unable to return database record"))
}

async fn killmail_time_calc(date_string: String) -> Result<String, reqwest::Error> {
    let dt: Vec<&str> = date_string.split("T").collect();
    let date = dt[0].replace("\"", "");

    let today = Utc::now();
    let todate = today.naive_utc();

    let pfs = chrono::NaiveDateTime::parse_from_str;

    let naive_dt = pfs(&date_string, "%Y-%m-%dT%H:%M:%SZ").expect("unable to parse kill date");

    let diff = todate - naive_dt;
    // let delta = diff.to_string();
    let hh = diff.num_hours();
    let mm = diff.num_minutes() % 60;
    let ss = diff.num_seconds() % 60;
    let delta = format!("{hh:02}h{mm:02}m{ss:02}s ago");

    Ok(delta)
}

async fn system_stats(system_name: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let system_id_lookup = name_lookup(system_name.to_string(), client.clone()).await?;

    println!("Looking up system name...");
    let system_id: String = system_id_lookup["systems"][0]["id"].to_string();
    println!("Looking up system id...");
    let system_zkb = get_system_kills(system_id.as_str(), client.clone()).await?;
    println!("Retrieving zkillboard for {system_name}...");
    let kills = get_num_kills(system_id.as_str(), client.clone()).await?;
    println!("Retrieving total number of ships killed in system in the last hour...");

    println!("Retrieving total number of NPCs killed in system in the last hour...");
    let system_jumps = get_jumps(system_id.as_str(), client.clone()).await?;
    println!("Retrieving total number of jumps in system in the last hour...");
    let system_gates = get_gates(system_id.as_str(), client.clone()).await?;
    println!("Determining number of available stargates...");
    let mut ccp_kills: Vec<CcpKillmail> = Vec::with_capacity(5);

    let mut kill_counter: i32 = 0;
    println!("Resolving most recent kills in system...");
    for key in system_zkb.iter() {
        let k = kill_resolve(key.killmail_id.to_string(), key.zkb.hash.to_string(), client.clone()).await?;
        ccp_kills.push(k);

        kill_counter += 1;
        if kill_counter == 5 {
            break
        }
    };

    let const_id = get_timer_const_id(system_id, client.clone()).await?;
    let region_id = get_timer_region_id(const_id.to_string(), client.clone()).await?;
    let region_name = get_timer_region_name(region_id.to_string(), client.clone()).await?;




    let mut ship = String::new();
    let mut char = String::new();
    let mut corp = String::new();
    let mut alli = String::new();

    let mut outputwrapper = Vec::new();
    let mut alli = String::new();
    for kill in ccp_kills {
        let mut output: Vec<String> = Vec::new();
        let ship = item_lookup(kill.victim.ship_type_id.expect("Can't find victim ship type").to_string(), client.clone()).await?;
        if kill.victim.character_id.is_none() {
            char = "None".to_string()
        } else {
            let resp: Value = legacy_item_lookup(kill.victim.character_id.unwrap().to_string(), client.clone()).await?;

            char = resp[0]["name"].to_string()

        }
        let corp = corp_info(kill.victim.corporation_id.to_string().as_str(), client.clone()).await?;
        let mut alli = String::new();

        match kill.victim.alliance_id {
            Some(i) => {
                let x = alliance_info(kill.victim.alliance_id.unwrap().to_string(), client.clone()).await?;
                alli = x.name

            }
            None => {
                "None";
            }
        }

        let killdelta = killmail_time_calc(kill.killmail_time).await?;
        output.push(killdelta);

        output.push(ship.to_string());
        output.push(char);
        output.push(corp.name);
        output.push(alli);
        outputwrapper.push(output);

    };
    println!("\nMost recent kill info for {system_name}:\n{:<15} {:<30} {:<25} {:<37} {:<25}",
             "Kill Age:",
             "Victim Ship:",
             "Victim Name:",
             "Victim Corp:",
             "Victim Alliance:");

    for kill in outputwrapper{

        println!("{:<15} {:<30} {:<25} {:<37} {:<25}",
            kill.get(0).unwrap().as_str().replace("\"", ""),
            kill.get(1).unwrap().as_str().replace("\"", ""),
            kill.get(2).unwrap().as_str().replace("\"", ""),
            kill.get(3).unwrap().as_str().replace("\"", ""),
            kill.get(4).unwrap().as_str().replace("\"", "")
        );


    }
    let mut npckills = String::new();
    let mut podkills = String::new();
    let mut shipkills = String::new();
    if kills.get(0).is_none() {
        npckills = 0.to_string()
    } else {
        npckills = kills.get(0).unwrap().to_string();
    };
    if kills.get(1).is_none() {
        podkills = 0.to_string()
    } else {
        podkills = kills.get(0).unwrap().to_string();
    };
    if kills.get(0).is_none() {
        shipkills = 0.to_string()
    } else {
        shipkills = kills.get(0).unwrap().to_string();
    };
    // let npckills = kills.get(0).unwrap().to_string();
    // let podkills = kills.get(1).unwrap().to_string();
    // let shipkills = kills.get(3).unwrap().to_string();

    println!("\nDotlan Map URL: https://evemaps.dotlan.net/map/{}/{}", region_name.replace(" ", "_"), system_name);

    println!("\nShips destroyed last hour: \t{:<30}", shipkills);
    println!("Capsules destroyed last hour: \t{:<30}", podkills);
    println!("NPCs destroyed last hour: \t{:<30}", npckills);
    println!("Jumps last hour: \t\t{:<30}", system_jumps);
    println!("Number of stargates in system: \t{:<30}\n", system_gates);
    Ok(())
}

async fn get_campaigns() -> Result<Campaigns, reqwest::Error> {
    let url = "https://esi.evetech.net/latest/sovereignty/campaigns/?datasource=tranquility";
    let response = reqwest::get(url).await?;
    let timers: Campaigns = response.json().await?;
    Ok(timers)

}

async fn get_system(system_id: &str) -> Result<SystemInfo, reqwest::Error> {
    let url = format!("https://esi.evetech.net/latest/universe/systems/{system_id}/");
    let response = reqwest::get(url).await?;
    let systeminfo: SystemInfo = response.json().await?;


    // let db_connect = db_connect().await;
    // let pool = db_connect.acquire().await.expect("Unable to create new pool connection");
    //
    // let item = sqlx::query!("SELECT  typeName FROM invTypes WHERE typeID IS ?", item_id)
    //     .fetch_one(&db_connect)
    //     .await
    //     .expect("Unable to query the database");
    //
    //
    //
    // Ok(item.typeName.expect("Unable to return database record"))
    Ok(systeminfo)
}

async fn get_const(const_id: &str) -> Result<ConstInfo, reqwest::Error> {
    let url = format!("https://esi.evetech.net/latest/universe/constellations/{const_id}/");
    let response = reqwest::get(url).await?;
    let constinfo: ConstInfo = response.json().await?;

    Ok(constinfo)
}

async fn get_region(const_id: &str) -> Result<RegionInfo, reqwest::Error> {
    let url = format!("https://esi.evetech.net/latest/universe/regions/{const_id}/");
    let response = reqwest::get(url).await?;

    let regioninfo: RegionInfo = response.json().await?;

    Ok(regioninfo)
}

async fn timer_time_calc(date_string: String) -> Result<String, reqwest::Error> {
    let dt: Vec<&str> = date_string.split("T").collect();
    let date = dt[0].replace("\"", "");

    let today = Utc::now();
    let todate = today.naive_utc();

    let pfs = chrono::NaiveDateTime::parse_from_str;

    let naive_dt = pfs(&date_string, "%Y-%m-%dT%H:%M:%SZ").expect("unable to parse kill date");

    let diff = naive_dt - todate;
    // let delta = diff.to_string();
    let hh = diff.num_hours();
    let mm = diff.num_minutes() % 60;
    let ss = diff.num_seconds() % 60;
    let delta = format!("{hh:02}h{mm:02}m{ss:02}s");

    Ok(delta)
}

async fn timers() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();


    let current_timers = get_campaigns().await?;
    let total_timers = current_timers.len();

    let mut output: Vec<String> = Vec::new();
    print!("Processing {total_timers} timers... ");
    io::stdout().flush().unwrap();
    for timer in current_timers.iter() {
        // let system_info: SystemInfo = get_system(timer.solar_system_id.to_string().as_str()).await?;
        // let const_info: ConstInfo = get_const(system_info.constellation_id.to_string().as_str()).await?;
        // let region_info: RegionInfo = get_region(const_info.region_id.to_string().as_str()).await?;

        let system_name = get_timer_solar_name(timer.solar_system_id.to_string(), client.clone()).await?;

        // let system_name = system_info.name;

        // let region_name = region_info.name;
        let region_id = get_timer_region_id(timer.constellation_id.to_string(), client.clone()).await?;
        let region_name = get_timer_region_name(region_id.to_string(), client.clone()).await?;
        let defender_value = alliance_info(timer.defender_id.to_string(), client.clone(), ).await?;
        let defender = defender_value.name;

        let timer_start = timer_time_calc(timer.start_time.to_string()).await?;


        output.push(format!("{:<20} {:<20} {:<20} {:<50} {:<20}",
                timer_start,
                 region_name,
                 system_name,
                 defender.replace("\"", ""),
                 timer.attackers_score.to_string().as_str()

        ));
        print!("*");
        io::stdout().flush().unwrap();
    }

    println!("\n{:<20} {:<20} {:<20} {:<50} {:<20}",
            "timer start:",
             "region:",
             "solar_system:",
             "defender:",
             "attacker_score:");
    for line in output.iter() {
        print!("{}\n", line);
        io::stdout().flush().unwrap();
    }
    println!("\n");
    Ok(())
}

async fn get_incursions() -> Result<Incursions, reqwest::Error> {
    let url = "https://esi.evetech.net/latest/incursions/?datasource=tranquility";
    let resp = reqwest::get(url).await?;
    let incursions: Incursions = resp.json().await?;
    Ok(incursions)
}

async fn incursions() -> Result<(), MyError> {
    let client = reqwest::Client::new();
    let mut output = Vec::new();
    let incursions: Incursions = get_incursions().await?;

    for incursion in incursions.iter() {
        // let const_info = get_const(incursion.constellation_id.to_string().as_str()).await?;
        let region_id = get_timer_region_id(incursion.constellation_id.to_string(), client.clone()).await?;
        let region_name = get_timer_region_name(region_id.to_string(), client.clone()).await?;
        let staging_system = get_timer_solar_name(incursion.staging_solar_system_id.to_string(), client.clone()).await?;
        let state = incursion.state.as_str();
        let has_boss = incursion.has_boss;
        let out_string = format!("{:<30} {:<20} {:<20} {:<20}",
            region_name,
            staging_system,
            state,
            has_boss);
        output.push(out_string)
    }
    println!("\n{:<30} {:<20} {:<20} {:<20}",
        "Region:",
        "Staging System:",
        "State:",
        "Has Boss:");
    for incursion in output.iter() {
        println!("{incursion}")
    }
    println!("\n");

    Ok(())
}

async fn get_sde_components() -> Result<(), MyError> {
// async fn get_sde_components() {

    // hold for sqldump conversions being enabled
    // let mut ss_present: bool = true;
    // let mut co_present: bool = true;
    // let mut re_present: bool = true;
    // let mut in_present: bool = true;
    //
    // ss_present = Path::new("mapSolarSystems.db").exists();
    // if ss_present == true {
    //     std::fs::remove_file("mapSolarSystems.db")?;
    // }
    // co_present = Path::new("mapConstellations.db").exists();
    // if ss_present == true {
    //     std::fs::remove_file("mapConstellations.db")?;
    // }
    // re_present = Path::new("mapRegions.db").exists();
    // if ss_present == true {
    //     std::fs::remove_file("mapRegions.db")?;
    // }
    // in_present = Path::new("invTypes.db").exists();
    // if ss_present == true {
    //     std::fs::remove_file("invTypes.db")?;
    // }
    // // std::fs::remove_file("mapSolarSystems.db")?;
    // // std::fs::remove_file("mapConstellations.db")?;
    // // std::fs::remove_file("mapRegions.db")?;
    // // std::fs::remove_file("invTypes.db")?;
    // let client = Client::new();
    //
    // let mut resp_solar = client.get("https://www.fuzzwork.co.uk/dump/latest/mapSolarSystems.sql.bz2")
    //     .header(USER_AGENT, "CLI EVE Utility Application by Sapporo Jones")
    //     .send()
    //     .await?
    //     .bytes_stream();
    //
    // let mut out_solar = std::fs::File::create("mapSolarSystems.sql.bz2")
    //     .expect("failed to create file");
    //
    //
    // while let Some(item) = resp_solar.next().await {
    //     std::io::copy(&mut item?.as_ref(), &mut out_solar).expect("Unable to write data");
    // }
    //
    // let mut resp_const = client.get("https://www.fuzzwork.co.uk/dump/latest/mapConstellations.sql.bz2")
    //     .header(USER_AGENT, "CLI EVE Utility Application by Sapporo Jones")
    //     .send()
    //     .await?
    //     .bytes_stream();
    //
    // let mut out_const = std::fs::File::create("mapConstellations.sql.bz2")
    //     .expect("failed to create file");
    //
    // while let Some(item) = resp_const.next().await {
    //     std::io::copy(&mut item?.as_ref(), &mut out_const).expect("Unable to write data");
    // }
    //
    // let mut resp_region = client.get("https://www.fuzzwork.co.uk/dump/latest/mapRegions.sql.bz2")
    //     .header(USER_AGENT, "CLI EVE Utility Application by Sapporo Jones")
    //     .send()
    //     .await?
    //     .bytes_stream();
    //
    // let mut out_region = std::fs::File::create("mapRegions.sql.bz2").expect("failed to create file");
    // while let Some(item) = resp_region.next().await {
    //     std::io::copy(&mut item?.as_ref(), &mut out_region).expect("Unable to write data");
    // }
    //
    // let mut resp_items = client.get("https://www.fuzzwork.co.uk/dump/latest/invTypes.sql.bz2")
    //     .header(USER_AGENT, "CLI EVE Utility Application by Sapporo Jones")
    //     .send()
    //     .await?
    //     .bytes_stream();
    //
    // let mut out_items = std::fs::File::create("invTypes.sql.bz2").expect("failed to create file");
    //
    // while let Some(item) = resp_items.next().await {
    //     std::io::copy(&mut item?.as_ref(), &mut out_items).expect("Unable to write data");
    // }
    //
    // let solar_path = r"./mapSolarSystems.sql.bz2";
    // let const_path = r"./mapConstellations.sql.bz2";
    // let region_path = r"./mapRegions.sql.bz2";
    // let items_path = r"./invTypes.sql.bz2";
    //
    // let mut sol_compressed_file = std::fs::File::open("mapSolarSystems.sql.bz2");
    // let mut sol_decompressed_output = std::fs::File::create("mapSolarSystems.sql");
    // let mut sol_reader = DecoderReader::new(sol_compressed_file?);
    // std::io::copy(&mut sol_reader, &mut sol_decompressed_output?).expect("Unable to write contents of file");
    //
    // let mut con_compressed_file = std::fs::File::open("mapConstellations.sql.bz2");
    // let mut con_decompressed_output = std::fs::File::create("mapConstellations.sql");
    // let mut con_reader = DecoderReader::new(con_compressed_file?);
    // std::io::copy(&mut con_reader, &mut con_decompressed_output?).expect("Unable to write contents of file");
    //
    // let mut reg_compressed_file = std::fs::File::open("mapRegions.sql.bz2");
    // let mut reg_decompressed_output = std::fs::File::create("mapRegions.sql");
    // let mut reg_reader = DecoderReader::new(reg_compressed_file?);
    // std::io::copy(&mut reg_reader, &mut reg_decompressed_output?).expect("Unable to write contents of file");
    //
    // let mut inv_compressed_file = std::fs::File::open("invTypes.sql.bz2");
    // let mut inv_decompressed_output = std::fs::File::create("invTypes.sql");
    // let mut inv_reader = DecoderReader::new(inv_compressed_file?);
    // std::io::copy(&mut inv_reader, &mut inv_decompressed_output?).expect("Unable to write contents of file");
    //
    // let ss_conn = Connection::open("mapSolarSystems.db")?;
    // let co_conn = Connection::open("mapConstellations.db")?;
    // let re_conn = Connection::open("mapRegions.db")?;
    // let in_conn = Connection::open("invTypes.db")?;
    //
    // let ss_sql = std::fs::read_to_string("mapSolarSystems.sql")?;
    // let co_sql = std::fs::read_to_string("mapConstellations.sql")?;
    // let re_sql = std::fs::read_to_string("mapRegions.sql")?;
    // let in_sql = std::fs::read_to_string("invTypes.sql")?;
    //
    // ss_conn.execute(&ss_sql, [])?;
    // co_conn.execute(&co_sql, [])?;
    // re_conn.execute(&re_sql, [])?;
    // in_conn.execute(&in_sql, [])?;

    let mut sde_present: bool = true;

    println!("Checking for previous SDE and removing if found...");

    sde_present = Path::new("sqlite-latest.sqlite").exists();
    if sde_present == true {
        std::fs::remove_file("sqlite-latest.sqlite").unwrap();
    }

    let client = Client::new();

    println!("Downloading latest SDE...");

    let mut resp_sde = client.get("https://www.fuzzwork.co.uk/dump/sqlite-latest.sqlite.bz2")
        .header(USER_AGENT, "CLI EVE Utility Application by Sapporo Jones")
        .send()
        .await?
        .bytes_stream();

    let mut out_sde = tokio::fs::File::create("sqlite-latest.sqlite.bz2").await
        .expect("failed to create file");


    while let Some(item) = resp_sde.next().await {
        tokio::io::copy(&mut item?.as_ref(), &mut out_sde).await.expect("Unable to write data");
    }

    println!("Decompressing SDE...");

    let sde_path = r"./sqlite-latest.sqlite.bz2";
    let mut sde_compressed_file = std::fs::File::open("sqlite-latest.sqlite.bz2");
    let mut sde_decompressed_output = std::fs::File::create("sqlite-latest.sqlite");
    let mut sde_reader = DecoderReader::new(sde_compressed_file?);
    std::io::copy(&mut sde_reader, &mut sde_decompressed_output.unwrap()).expect("Unable to write contents of SDE");

    println!("Cleaning up...");

    sde_present = Path::new("sqlite-latest.sqlite.bz2").exists();
    if sde_present == true {
        std::fs::remove_file("sqlite-latest.sqlite.bz2").unwrap();
    }

    println!("Done!");

    Ok(())
}