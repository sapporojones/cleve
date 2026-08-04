#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(unused)]

mod first_order;
mod lookups;
mod helpers;


use clap::{Parser, Subcommand};
use futures_util::{StreamExt, TryStreamExt};
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::io::Write;
use std::string::String;
use std::time::SystemTime;
use thiserror::Error;
use serde_jsonlines::AsyncJsonLinesReader;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::fs::File;
use clap::builder::TypedValueParser;
use zip_extensions::*;

#[derive(Error, Diagnostic, Debug)]
enum MyError {
    // #[error("Assertion failed")]
    // AE(#[from] tokio::task::),
    #[error("IO Error")]
    IO(#[from] std::io::Error),
    #[error("FE Error")]
    FE(#[from] std::fmt::Error),
    #[error("Reqwest Error")]
    RE(#[from] reqwest::Error),
    // #[error("SQLx Error")]
    // SqE(#[from] sqlx::Error),
    #[error("Other Error")]
    Custom(String),
}

async fn db_connect() -> Pool<Sqlite> {
    let options = "./eve-sde-latest-jsonl.sqlite";
    let pool = SqlitePool::connect(options)
        .await
        .expect("Unable to connect to database");
    pool
}

#[derive(Serialize, Deserialize, Debug)]
struct Skyhooks {
    pub skyhooks: Vec<Hooks>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Hooks {
    pub planet_id: i64,
    pub solar_system_id: i64,
    pub theft_vulnerability: TheftVulnerability,

}

#[derive(Serialize, Deserialize, Debug)]
struct TheftVulnerability {
    pub end: String,
    pub start: String,
}

#[derive(Serialize, Deserialize)]
struct Langstruct {
    pub de: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub en: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub es: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ja: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ko: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ru: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zh: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SDETypestruct {
    pub _key: i64,
    // #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "basePrice", skip_serializing_if = "Option::is_none")]
    pub base_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Langstruct>,
    #[serde(rename = "groupID")]
    pub group_id: i64,
    pub name: Langstruct,
    #[serde(rename = "portionSize")]
    pub portion_size: i64,
    pub published: bool,
    #[serde(rename = "raceID", skip_serializing_if = "Option::is_none")]
    pub race_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
}



#[derive(Serialize, Deserialize)]
struct SDESystemStruct {
    pub _key: i64,
    #[serde(rename = "constellationID")]
    pub constellation_id: i64,
    #[serde(rename = "disallowedAnchorCategories", skip_serializing_if = "Option::is_none")]
    pub disallowed_anchor_categories: Option<Vec<i64>>,
    pub name: Langstruct,
    pub position: Position,
    pub radius: f64,
    #[serde(rename = "regionID", skip_serializing_if = "Option::is_none")]
    pub region_id: Option<i64>,
    #[serde(rename = "securityStatus")]
    pub security_status: f64,
}


#[derive(Serialize, Deserialize)]
struct SDEConstStruct {
    pub _key: i64,
    pub name: Langstruct,
    pub position: Position,
    #[serde(rename = "regionID")]
    pub region_id: i64,
    #[serde(rename = "solarSystemIDs")]
    pub solar_system_ids: Vec<i64>,
    #[serde(rename = "wormholeClassID", skip_serializing_if = "Option::is_none")]
    pub wormhole_class_id: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct SDERegionStruct {
    pub _key: i64,
    #[serde(rename = "constellationIDs")]
    pub constellation_ids: Vec<i64>,
    pub name: Langstruct,
    #[serde(rename = "nebulaID")]
    pub nebula_id: i64,
    pub position: Position,
    #[serde(rename = "wormholeClassID", skip_serializing_if = "Option::is_none")]
    pub wormhole_class_id: Option<i64>,
}



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
    #[serde(rename = "locationID", skip_serializing_if = "Option::is_none")]
    pub location_id: Option<i64>,
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

pub type EsiSystemKills = Vec<SystemKills>;

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemKills {
    pub npc_kills: i64,
    pub pod_kills: i64,
    pub ship_kills: i64,
    pub system_id: i64,
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

pub type CorpHistory = Vec<CorpHist>;

#[derive(Serialize, Deserialize)]
pub struct CorpHist {
    corporation_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_deleted: Option<bool>,
    record_id: i64,
    start_date: String,
}

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
// #[command(value_delimiter = " ")]
struct Cli {
    // #[command(value_delimiter = true)]

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corporation_id: Option<i64>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corporation_id: Option<i64>,
    pub damage_taken: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction_id: Option<i64>,
    pub items: Vec<Option<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<KillPosition>,
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
    System { system_name: String },
    /// Retrieve current status of the Tranquility server
    Status,
    /// Returns information about current sov timers (SOON TO BE DEPRECATED)
    Timers,
    /// Returns information about upcoming Skyhook vulnerabilities around the game using FC's default sort
    Skyhooks,
    /// Shows only upcoming Skyhook vulnerabilities matching a given region name
    Hooks { region_name: String },
    /// Update SDE components
    Update,
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let start = SystemTime::now();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Travel) => {
            first_order::evescout().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Pilot { character_name }) => {
            first_order::shlookup(character_name.as_str());


            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Thera) => {
            first_order::thera().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Turnur) => {
            first_order::turnur().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Status) => {
            first_order::status().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::System { system_name }) => {
            first_order::system_stats(system_name).await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Incursions) => {
            first_order::incursions().await.expect("Unable to fetch incursion data");

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Update) => {
            let _ = first_order::get_sde_components().await;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Timers) => {
            first_order::timers().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Skyhooks) => {
            first_order::get_upcoming_skyhooks().await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        Some(Commands::Hooks { region_name }) => {
            first_order::get_hooks_by_region(region_name.to_string()).await?;

            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Completed in {} seconds.", duration.as_secs_f64());
        }
        None => {
            println!(
                "No command specified.  Please supply a command or re-run with --help for help."
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::testing::TestTermination;
    use futures_util::TryStreamExt;
    use crate::first_order::{system_stats, timers};

    // Check if ESI is responding by querying status and expecting a 200 status code
    #[tokio::test]
    async fn check_esi() -> Result<(), MyError> {
        let url = "https://esi.evetech.net/latest/status/?datasource=tranquility";
        let status_response = reqwest::get(url).await?;

        if status_response.status().is_success() {
            Ok(())
        } else {
            Err(panic!(
                "ESI response code was {}",
                status_response.status().to_string()
            ))
        }
    }

    // Check if ESI server status endpoint is responding by querying status and expecting a 200 status code
    #[tokio::test]
    async fn check_api_endpoint_tq_status() -> Result<(), MyError> {
        let url = "https://esi.evetech.net/latest/status/?datasource=tranquility";
        let status_response = reqwest::get(url).await?;

        if status_response.status().is_success() {
            Ok(())
        } else {
            Err(panic!(
                "ESI response code was {}",
                status_response.status().to_string()
            ))
        }
    }

    // Check if eve scout thera endpoint is responding by querying status and expecting a 200 status code
    #[tokio::test]
    async fn check_api_endpoint_thera() -> Result<(), MyError> {
        let status_response = reqwest::get(
            "https://api.eve-scout.com//v2/public/signatures?system_name=thera",
        ).await?;
        if status_response.status().is_success() {
            Ok(())
        } else {
            Err(panic!(
                "ESI response code was {}",
                status_response.status().to_string()
            ))
        }
    }

    // Check if eve scout turnur endpoint is responding by querying status and expecting a 200 status code
    #[tokio::test]
    async fn check_api_endpoint_turnur() -> Result<(), MyError> {
        let status_response = reqwest::get(
            "https://api.eve-scout.com//v2/public/signatures?system_name=turnur",
        ).await?;
        if status_response.status().is_success() {
            Ok(())
        } else {
            Err(panic!(
                "ESI response code was {}",
                status_response.status().to_string()
            ))
        }
    }

    // Check if ESI incursions endpoint is responding by querying and expecting a 200 status code
    #[tokio::test]
    async fn check_api_endpoint_incursions() -> Result<(), MyError> {
        let url = "https://esi.evetech.net/latest/incursions/?datasource=tranquility";
        let status_response = reqwest::get(url).await?;

        if status_response.status().is_success() {
            Ok(())
        } else {
            Err(panic!(
                "ESI response code was {}",
                status_response.status().to_string()
            ))
        }
    }

    // Check if ESI campaigns endpoint is responding
    #[tokio::test]
    async fn check_api_endpoint_campaigns() -> Result<(), MyError> {
        let url = "https://esi.evetech.net/latest/sovereignty/campaigns/?datasource=tranquility";
        let status_response = reqwest::get(url).await?;

        if status_response.status().is_success() {
            Ok(())
        } else {
            Err(panic!(
                "ESI response code was {}",
                status_response.status().to_string()
            ))
        }
    }

    // Check if the timers function runs successfully
    #[tokio::test]
    async fn check_feature_timers() -> Result<(), Box<dyn std::error::Error>> {
        if timers().await.is_success() {
            Ok(())
        } else {
            Err(panic!("Function test failed."))
        }
    }

    // Check if the shlookup function (the pilot command) runs successfully
    #[tokio::test]
    async fn check_feature_shlookup() -> Result<(), Box<dyn std::error::Error>> {
        if first_order::shlookup("Sapporo Jones").await.is_success() {
            Ok(())
        } else {
            Err(panic!("Function test failed."))
        }
    }

    // Check if the system status function (the system command) runs successfully
    #[tokio::test]
    async fn check_feature_system() -> Result<(), Box<dyn std::error::Error>> {
        if system_stats("Jita").await.is_success() {
            Ok(())
        } else {
            Err(panic!("Function test failed."))
        }
    }
    // Check if JSONLines parsing works for inventory types (types.jsonl)
    #[tokio::test]
    async fn check_types_parsing() -> Result<(), Box<dyn std::error::Error>> {
        let fp = BufReader::new(File::open("./sde/types.jsonl").await?);
        let reader = AsyncJsonLinesReader::new(fp);
        let typesde = reader
            .read_all::<SDETypestruct>()
            .try_collect::<Vec<_>>()
            .await?;
        let testvar = 3125; // Hail, Eris!
        let mut foundvar = false;
        for x in typesde {
            if x._key == 3125 {
                foundvar = true;
            }
        }
        if !foundvar {
            panic!("Something failed parsing the SDE JSONL");
        }
        Ok(())
    }
    #[tokio::test]
    async fn check_mapsolarsystems_id() -> Result<(), Box<dyn std::error::Error>> {
        let fp = BufReader::new(File::open("./sde/mapSolarSystems.jsonl").await?);
        let reader = AsyncJsonLinesReader::new(fp);
        let typesde = reader
            .read_all::<SDESystemStruct>()
            .try_collect::<Vec<_>>()
            .await?;
        let testvar = 30000142; // Jita
        let mut foundvar = false;
        for x in typesde {
            if x._key == 30000142 {
                foundvar = true;
            }
        }
        if !foundvar {
            panic!("Something failed parsing the SDE JSONL");
        }
        Ok(())
    }
    #[tokio::test]
    async fn check_mapconst_id() -> Result<(), Box<dyn std::error::Error>> {
        let fp = BufReader::new(File::open("./sde/mapConstellations.jsonl").await?);
        let reader = AsyncJsonLinesReader::new(fp);
        let typesde = reader
            .read_all::<SDEConstStruct>()
            .try_collect::<Vec<_>>()
            .await?;
        let testvar = 20000665; // Pegasus constellation - home of extremely valuable gas clouds
        let mut foundvar = false;
        for x in typesde {
            if x._key == 20000665 {
                foundvar = true;
            }
        }
        if !foundvar {
            panic!("Something failed parsing the SDE JSONL");
        }
        Ok(())
    }
    #[tokio::test]
    async fn check_mapregions_id() -> Result<(), Box<dyn std::error::Error>> {
        let fp = BufReader::new(File::open("./sde/mapRegions.jsonl").await?);
        let reader = AsyncJsonLinesReader::new(fp);
        let typesde = reader
            .read_all::<SDERegionStruct>()
            .try_collect::<Vec<_>>()
            .await?;
        let testvar = 10000002; // The Forge region
        let mut foundvar = false;
        for x in typesde {
            if x._key == 10000002 {
                foundvar = true;
            }
        }
        if !foundvar {
            panic!("Something failed parsing the SDE JSONL");
        }
        Ok(())
    }
}
