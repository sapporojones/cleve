use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};
use chrono::Utc;
use serde_json::{json, Value};
use std::time::SystemTime;

type Wrapper = Vec<Hole>;

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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// For listing public travel wormhole routes from Thera or Turnur
    Travel,
    /// For Thera specific wormhole information
    Thera,
    /// For Turnur specific wormhole information
    Turnur,
    /// For information about a character
    Shlookup {
        /// Name of character to lookup, if character name contains spaces quotation marks must be used
        character_name: String,
    },

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
        Some(Commands::Shlookup { character_name }) =>{
            shlookup(character_name.as_str());

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
        None => {
            println!("No command specified.  Please supply a command or re-run with --help for help.");
        }
    }
    Ok(())
}

async fn evescout() -> Result<(), reqwest::Error> {
    let rthera = reqwest::get("https://api.eve-scout.com//v2/public/signatures?system_name=thera").await?;
    let thera: Wrapper = rthera.json().await?;
    let rturnur = reqwest::get("https://api.eve-scout.com//v2/public/signatures?system_name=turnur").await?;
    let turnur: Wrapper = rturnur.json().await?;

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
    let rthera = reqwest::get("https://api.eve-scout.com//v2/public/signatures?system_name=thera").await?;
    let thera: Wrapper = rthera.json().await?;
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
    let rturnur = reqwest::get("https://api.eve-scout.com//v2/public/signatures?system_name=turnur").await?;
    let turnur: Wrapper = rturnur.json().await?;
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

fn shlookup(char_name: &str) {

    // // test char id:
    // // sappo = 772506501
    // // billy = 1826057122
    // // d1ck = 749151393
    // // comment out below for release version
    // let char_id: &str = "772506501";
    // uncomment below to accept char id from user via command line args
    let ci = char_name;
    let char_id = char_search(ci);
    let p: Value = public_info(char_id.as_str());
    let corpid: Value = p["corporation_id"].clone();
    let c: Value = corp_info(corpid.to_string().as_str());

    let alliance_info = if c["alliance_id"].is_null() {
        json!({"Alliance":  "Corporation is not a member of an alliance."})
    } else {
        alice_info(c["alliance_id"].to_string())
    };

    let zs: Value = get_zkb_stats(char_id.clone());

    let mr_kill: Value = get_mr_kill_info(char_id.clone().to_string());
    let mr_loss: Value = get_mr_loss_info(char_id.clone().to_string());
    // println!("{}", mr_kill["victim"]["ship_type_id"].to_string().replace("\"", ""));
    let killed_with_j: Value = item_lookup(
        mr_kill["victim"]["ship_type_id"]
            .to_string()
            .replace("\"", ""),
    );
    let killed_with: String = killed_with_j[0]["name"].to_string().replace("\"", "");
    let lost_ship_j: Value = item_lookup(
        mr_loss["victim"]["ship_type_id"]
            .to_string()
            .replace("\"", ""),
    );
    // println!("{}", lost_ship_j.to_string());
    let lost_ship: String = lost_ship_j[0]["name"].to_string().replace("\"", "");

    // println!("\n \n");
    println!("\n \nBasic info:");
    println!("Name: {}", p["name"].to_string().replace("\"", ""));
    let bday_raw: String = p["birthday"].to_string().replace("\"", "");
    let bday_clean: String = date_parse(&bday_raw);

    println!("Birthday: {}", bday_clean);


    let sec_status: String = p["security_status"].to_string();
    println!("Security Status: {:}", &sec_status.as_str().replace("\"", ""));

    println!(
        "\nCorporation: {} [{}]",
        c["name"].to_string().replace("\"", ""),
        c["ticker"].to_string().replace("\"", "")
    );
    // println!("Ticker: {}", c["ticker"]);
    println!("Corporation members: {}", c["member_count"]);
    println!("Corporation tax rate: {}", c["tax_rate"]);

    let corp_bday_raw: String = c["date_founded"].to_string().replace("\"", "");
    let corp_bday: String = date_parse(&corp_bday_raw);
    println!("Corporation founded on: {}", corp_bday);
    println!(
        "Corporation evewho: https://evewho.com/corporation/{}",
        p["corporation_id"]
    );

    if c["alliance_id"].is_null() {
        println!("\nAlliance:  Corporation is not a member of an alliance.")
    } else {
        // println!("Alliance: {} - {}", a["name"], a["ticker"])
        println!(
            "\nAlliance: {} [{}]",
            alliance_info["name"].to_string().replace("\"", ""),
            alliance_info["ticker"].to_string().replace("\"", "")
        );

        let alliance_bday_raw: String = alliance_info["date_founded"].to_string().replace("\"", "");
        let alliance_bday: String = date_parse(&alliance_bday_raw);

        println!("Alliance founded on: {}", alliance_bday);
        println!(
            "Alliance evewho: https://evewho.com/alliance/{}",
            c["alliance_id"]
        )
    }

    println!("\n\nZKB Stats:");
    println!(
        "Character Zkb: https://zkillboard.com/character/{}/",
        char_id
    );
    println!("\nAlltime kills: {}", zs["shipsDestroyed"]);
    println!("Alltime losses: {}", zs["shipsLost"]);
    println!("Solo kills: {}", zs["soloKills"]);
    println!("Solo losses: {}", zs["soloLosses"]);

    let kt = mr_kill["killmail_time"].to_string();
    let kt_clean: String = date_parse(&kt);
    let kill_diff: i64 = date_calc(kt.clone());
    println!(
        "\nMost recently killed a(n) {} on {} which was {} days ago",
        killed_with, &kt_clean, kill_diff
    );

    let lt = mr_loss["killmail_time"].to_string();
    let lt_clean: String = date_parse(&lt);
    let loss_diff = date_calc(lt.clone());
    println!(
        "Most recently lost a(n) {} on {} which was {} days ago",
        lost_ship, &lt_clean, loss_diff
    );

    println!("\n \n");


}

fn char_search(char_name: &str) -> String {
    let ps = format!("[{:?}]", char_name);
    let payload = json!(ps);
    let pl = payload.as_str().unwrap();
    println!("Searching for {:?}...", char_name);

    let url = "https://esi.evetech.net/latest/universe/ids/?datasource=tranquility&language=en";
    let res: Value = ureq::post(url)
        .set("Accept", "application/json")
        .set("Accept-Language", "en")
        .set("Content-Type", "application/json")
        .set("Cache-Control", "no-cache")
        .send_string(pl)
        .expect("there was an error handling the response from ccp")
        .into_json()
        .expect("couldn't coerce search result to json");

    let rj = &res["characters"];
    let rout: Value = json!(rj);

    let char_id: &str = &rout[0]["id"].to_string();
    println!("{:?} found as {:?}...", char_name, char_id);
    char_id.to_string()
}

fn public_info(char_id: &str) -> Value {
    println!("Fetching public info...");
    let p_url: String = format!(
        "https://esi.evetech.net/latest/characters/{char_id}/?datasource=tranquility"
    );

    let ps: String = ureq::get(&p_url)
        .call()
        .expect("pubinfo call failed")
        .into_string()
        .expect("couldn't convert pubinfo to string");
    let p: Value = serde_json::from_str(ps.as_str()).expect("couldn't convert pubinfo to json");

    p
}

fn corp_info(corporation_id: &str) -> Value {
    println!("Fetching corporation info...");
    let c_url: String = format!(
        "https://esi.evetech.net/latest/corporations/{}/?datasource=tranquility",
        corporation_id
    );

    let cs: String = ureq::get(&c_url)
        .call()
        .expect("this shouldn't fail")
        .into_string()
        .expect("couldn't coerce to string");

    let c: Value = serde_json::from_str(cs.as_str()).expect("couldn't convert to json");

    c
}

fn alice_info(corporation_id: String) -> Value {
    println!("Fetching alliance information...");
    let a_url: String = format!(
        "https://esi.evetech.net/latest/alliances/{}/?datasource=tranquility",
        corporation_id
    );
    let a_s: String = ureq::get(&a_url)
        .call()
        .expect("Corporation is not in an alliance")
        .into_string()
        .expect("");

    let a: Value =
        serde_json::from_str(a_s.as_str()).expect("couldn't convert alliance info to json");
    // println!("Alliance: {} - {}", a["name"], a["ticker"]);

    a
}

fn get_mr_kill_info(char_id: String) -> Value {
    println!("Fetching most recent kill data...");
    let url = format!("https://zkillboard.com/api/kills/characterID/{}/", char_id);
    let kills = ureq::get(&url)
        .call()
        .expect("couldn't retrieve zkb data for this character for some reason...")
        .into_string()
        .expect("couldn't convert zkb info to json");

    let zkb: Value =
        serde_json::from_str(kills.as_str()).expect("couldn't convert zkb data to json");

    let mr_id: String = zkb[0]["killmail_id"].to_string();
    let mr_hash: String = zkb[0]["zkb"]["hash"].to_string().replace("\"", "");
    // println!("{} {} ", mr_id, mr_hash);

    let mr_kill: Value = kill_resolve(mr_id.to_string(), mr_hash.to_string());

    mr_kill
}

fn get_mr_loss_info(char_id: String) -> Value {
    println!("Fetching most recent loss data...");
    let url = format!("https://zkillboard.com/api/losses/characterID/{}/", char_id);
    let losses = ureq::get(&url)
        .call()
        .expect("couldn't retrieve zkb data for this character for some reason...")
        .into_string()
        .expect("couldn't convert zkb info to json");

    let zkb: Value =
        serde_json::from_str(losses.as_str()).expect("couldn't convert zkb data to json");

    let mr_id: String = zkb[0]["killmail_id"].to_string();
    let mr_hash: String = zkb[0]["zkb"]["hash"].to_string().replace("\"", "");

    let mr_loss: Value = kill_resolve(mr_id.to_string(), mr_hash.to_string());

    mr_loss
}

fn get_zkb_stats(char_id: String) -> Value {
    println!("Fetching zkill stats data...");
    let url = format!("https://zkillboard.com/api/stats/characterID/{}/", char_id);
    let st = ureq::get(&url)
        .call()
        .expect("couldn't retrieve zkb data for this character for some reason...")
        .into_string()
        .expect("couldn't convert json output to string");

    let zkb = serde_json::from_str(st.as_str()).expect("couldn't convert zkb data to json");

    zkb
}

fn kill_resolve(kill_id: String, kill_hash: String) -> Value {
    let url = format!(
        "https://esi.evetech.net/latest/killmails/{}/{}/?datasource=tranquility",
        kill_id, kill_hash
    );
    // println!("{} {} {}", kill_id, kill_hash, url);
    let resp = ureq::get(&url)
        .call()
        .expect("received an error while communicating with ccp")
        .into_string()
        .expect("couldn't convert return data to string");

    let kill_info: Value =
        serde_json::from_str(resp.as_str()).expect("couldn't convert response data to json");

    kill_info
}

fn date_calc(date_string: String) -> i64 {
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

    days
}

fn date_parse(date_string: &String) -> String {
    let dt: Vec<&str> = date_string.split("T").collect();
    let date = dt[0].replace("\"", "");

    let pfs = chrono::NaiveDate::parse_from_str;

    let naive_dt = pfs(&date, "%Y-%m-%d").expect("unable to parse kill date");
    naive_dt.to_string()
}

fn item_lookup(item_id: String) -> Value {
    let ps = format!("[{}]", item_id);
    let payload = json!(ps);
    let pl = payload.as_str().unwrap();
    // println!("{}", pl);
    let url = "https://esi.evetech.net/latest/universe/names/?datasource=tranquility&language=en";
    let res: Value = ureq::post(url)
        .set("Accept", "application/json")
        .set("Accept-Language", "en")
        .set("Content-Type", "application/json")
        .set("Cache-Control", "no-cache")
        .send_string(&pl)
        .expect("there was an error handling the response from ccp")
        .into_json()
        .expect("couldn't coerce search result to json");
    res
}

