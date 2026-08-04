use reqwest::Client;
use serde_json::{json, Value};
use crate::{AllianceInfo, Campaigns, CcpKillmail, CharInfo, ConstInfo, CorpHistory, CorpInfo, EsiSystemKills, Incursions, MyError, RegionInfo, SysJumps, SystemInfo, SystemZkb};

pub async fn char_search(char_name: &str, client: Client) -> Result<String, reqwest::Error> {
    let payloadstring = format!("[{:?}]", char_name);
    // println!("Searching for {:?}...", char_name);

    let url = "https://esi.evetech.net/latest/universe/ids/?datasource=tranquility&language=en";

    let resp = client.post(url).body(payloadstring).send().await?;
    let lookup: Value = resp.json().await?;
    let char_id = lookup["characters"][0]["id"].to_string();

    println!("{:?} found as {:?}...", char_name, char_id);
    Ok(char_id)
}

pub async fn public_info(char_id: &str, client: Client) -> Result<CharInfo, reqwest::Error> {
    // println!("Fetching public info...");
    let url: String =
        format!("https://esi.evetech.net/latest/characters/{char_id}/?datasource=tranquility");

    let publicinfo_response = client.get(&url).send().await?;
    let p: CharInfo = publicinfo_response.json().await?;

    Ok(p)
}

async fn get_corp_history(char_id: String, client: Client) -> Result<CorpHistory, reqwest::Error> {
    let url = format!("https://esi.evetech.net/characters/{}/corporationhistory", char_id);
    let hist_response = client.get(url).send().await?;
    let corp_history: CorpHistory = hist_response.json().await?;
    Ok(corp_history)
}

pub async fn corp_info(corporation_id: &str, client: Client) -> Result<CorpInfo, reqwest::Error> {
    // println!("Fetching corporation info...");
    let url: String = format!(
        "https://esi.evetech.net/latest/corporations/{}/?datasource=tranquility",
        corporation_id
    );

    let corp_response = client.get(url).send().await?;
    let corp_info: CorpInfo = corp_response.json().await?;
    Ok(corp_info)
}

pub async fn alliance_info(
    corporation_id: String,
    client: Client,
) -> Result<AllianceInfo, reqwest::Error> {
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

    let mr_kill: CcpKillmail =
        kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;

    Ok(mr_kill)
}

async fn get_kill_info(char_id: String, client: Client) -> Result<CcpKillmail, reqwest::Error> {
    // println!("Fetching most recent kill data...");
    let url = format!("https://zkillboard.com/api/kills/characterID/{}/", char_id);

    let kills_response = client.get(url).send().await?;
    let zkb: Value = kills_response.json().await?;

    let mr_id: String = zkb[0]["killmail_id"].to_string();
    let mr_hash: String = zkb[0]["zkb"]["hash"].to_string().replace("\"", "");

    let mr_kill: CcpKillmail =
        kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;

    Ok(mr_kill)
}

async fn get_loss_info(char_id: String, client: Client) -> Result<CcpKillmail, reqwest::Error> {
    // println!("Fetching most recent kill data...");
    let url = format!("https://zkillboard.com/api/losses/characterID/{}/", char_id);

    let kills_response = client.get(url).send().await?;
    let zkb: Value = kills_response.json().await?;

    let mr_id: String = zkb[0]["killmail_id"].to_string();
    let mr_hash: String = zkb[0]["zkb"]["hash"].to_string().replace("\"", "");

    let mr_kill: CcpKillmail =
        kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;

    Ok(mr_kill)
}

async fn get_mr_loss_info(char_id: String, client: Client) -> Result<CcpKillmail, reqwest::Error> {
    // println!("Fetching most recent loss data...");
    let url = format!("https://zkillboard.com/api/losses/characterID/{}/", char_id);

    let losses = client.get(url).send().await?;
    let zkb: Value = losses.json().await?;

    let mr_id: String = zkb[0]["killmail_id"].to_string();
    let mr_hash: String = zkb[0]["zkb"]["hash"].to_string().replace("\"", "");

    let mr_loss: CcpKillmail =
        kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;

    Ok(mr_loss)
}

pub async fn get_zkb_stats(char_id: String, client: Client) -> Result<Value, reqwest::Error> {
    println!("Fetching zkill stats data...");
    let url = format!("https://zkillboard.com/api/stats/characterID/{}/", char_id);

    let response = client.get(url).send().await?;

    let zkb = response.json().await?;

    Ok(zkb)
}

pub async fn kill_resolve(
    kill_id: String,
    kill_hash: String,
    client: Client,
) -> Result<CcpKillmail, reqwest::Error> {
    let url = format!(
        "https://esi.evetech.net/latest/killmails/{}/{}/?datasource=tranquility",
        kill_id, kill_hash
    );

    let response = client.get(url).send().await?;

    let kill_info: CcpKillmail = response.json().await?;

    Ok(kill_info)
}

pub async fn item_lookup(item_id: String, client: Client) -> Result<String, MyError> {
    let db_connect = crate::db_connect().await;
    let pool = db_connect
        .acquire()
        .await
        .expect("Unable to create new pool connection");

    let item = sqlx::query!("SELECT  name_en FROM types WHERE _key IS ?", item_id)
        .fetch_one(&db_connect)
        .await
        .expect("Unable to query the database");

    Ok(item.name_en.expect("Unable to return database record"))

    // no more dbs so converting to jsonl
    // let fp = BufReader::new(File::open("./sde/types.jsonl").await?);
    // let reader = AsyncJsonLinesReader::new(fp);
    // let typesde = reader
    //     .read_all::<SDETypestruct>()
    //     .try_collect::<Vec<_>>()
    //     .await?;
    // let itemid: i64 = item_id.parse().unwrap();
    // // let mut foundvar = false;
    // let mut name = String::new();
    // for x in typesde {
    //     if x._key == itemid {
    //         name = x.name.en.expect("couldn't locate itemid in types.jsonl");
    //     }
    // }
    // Ok(name)
}

pub async fn legacy_item_lookup(item_id: String, client: Client) -> Result<Value, reqwest::Error> {
    let ps = format!("[{}]", item_id);
    let payload = json!(ps);
    let pl = payload.as_str().unwrap();

    let url = "https://esi.evetech.net/latest/universe/names/?datasource=tranquility&language=en";

    let response = client.post(url).body(ps).send().await?;

    let res: Value = response.json().await?;

    Ok(res)
}

// async fn name_lookup(item_name: String, client: Client, dbconnect: &Pool<SqlitePool>) -> Result<Value, reqwest::Error> {
pub async fn name_lookup(item_name: String, client: Client) -> Result<Value, reqwest::Error> {
    let ps = format!("[\"{item_name}\"]");

    let url = "https://esi.evetech.net/latest/universe/ids/?datasource=tranquility&language=en";

    let response = client.post(url).body(ps).send().await?;

    let lookup: Value = response.json().await?;

    Ok(lookup)
}

pub async fn get_jumps(system_id: &str, client: Client) -> Result<String, reqwest::Error> {
    let url = "https://esi.evetech.net/latest/universe/system_jumps/?datasource=tranquility";
    let sysjumps = client.get(url).send().await?;
    let jumpstruct: SysJumps = sysjumps.json().await?;

    let mut j: i64 = 0;
    for key in jumpstruct.iter() {
        if key.system_id.to_string().as_str() == system_id {
            j = key.ship_jumps;
        };
    }
    let jumps: String = j.to_string();
    Ok(jumps.to_string())
}

pub async fn get_gates(system_id: &str, client: Client) -> Result<String, reqwest::Error> {
    let url = format!("https://esi.evetech.net/latest/universe/systems/{system_id}/");
    let gate_response = client.get(url).send().await?;
    let gates: SystemInfo = gate_response.json().await?;
    let num_gates = gates.stargates.len().to_string();
    Ok(num_gates)
}

pub async fn get_num_kills(system_id: &str, client: Client) -> Result<Vec<String>, reqwest::Error> {
    let url = "https://esi.evetech.net/latest/universe/system_kills/?datasource=tranquility";
    let kills_response = client.get(url).send().await?;
    let killsj: EsiSystemKills = kills_response.json().await?;

    let mut kills_vec: Vec<String> = Vec::new();

    for key in killsj.iter() {
        if key.system_id.to_string().as_str() == system_id {
            kills_vec.push(key.npc_kills.to_string());
            kills_vec.push(key.pod_kills.to_string());
            kills_vec.push(key.ship_kills.to_string());
        };
    }

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
    }
    let kills: String = k.to_string();
    Ok(kills)
}

pub async fn get_system_kills(system_id: &str, client: Client) -> Result<SystemZkb, reqwest::Error> {
    let url = format!("https://zkillboard.com/api/solarSystemID/{system_id}/");
    let zkbsys_response = client.get(url).send().await?;

    let zkbsysj: SystemZkb = zkbsys_response.json().await?;
    Ok(zkbsysj)
}

async fn get_timer_solar_id(system_id: String, client: Client) -> Result<i64, MyError> {
    let db_connect = crate::db_connect().await;
    let pool = db_connect
        .acquire()
        .await
        .expect("Unable to create new pool connection");

    let system = sqlx::query!(
        "SELECT _key FROM mapSolarSystems WHERE name_en IS ?",
        system_id
    )
    .fetch_one(&db_connect)
    .await
    .expect("Unable to query the database");

    Ok(system._key.expect("couldn't retrieve value"))

    // let fp = BufReader::new(File::open("./sde/mapSolarSystems.jsonl").await?);
    // let reader = AsyncJsonLinesReader::new(fp);
    // let typesde = reader
    //     .read_all::<SDESystemStruct>()
    //     .try_collect::<Vec<_>>()
    //     .await?;
    // // let system: i64 = system_id.parse().unwrap();
    // // let mut foundvar = false;
    // let mut sys_id = 0;
    // for x in typesde {
    //     if x.name.en == Some(system_id.clone()) {
    //         sys_id = x._key
    //     }
    // }
    // Ok(sys_id)
}

pub async fn get_timer_solar_name(system_id: String, client: Client) -> Result<String, MyError> {
    let db_connect = crate::db_connect().await;
    let pool = db_connect
        .acquire()
        .await
        .expect("Unable to create new pool connection");

    let system = sqlx::query!(
        "SELECT name_en FROM mapSolarSystems WHERE _key IS ?",
        system_id
    )
    .fetch_one(&db_connect)
    .await
    .expect("Unable to query the database");

    Ok(system
        .name_en
        .expect("Unable to return database record"))

    // let fp = BufReader::new(File::open("./sde/mapSolarSystems.jsonl").await?);
    // let reader = AsyncJsonLinesReader::new(fp);
    // let typesde = reader
    //     .read_all::<SDESystemStruct>()
    //     .try_collect::<Vec<_>>()
    //     .await?;
    // let system: i64 = system_id.parse().unwrap();
    // // let mut foundvar = false;
    // let mut name = String::new();
    // for x in typesde {
    //     if x._key == system {
    //         name = x.name.en.expect("couldn't locate itemid in types.jsonl");
    //     }
    // }
    // Ok(name)
}

async fn get_timer_const_id(system_id: String, client: Client) -> Result<i64, MyError> {
    let db_connect = crate::db_connect().await;
    let pool = db_connect
        .acquire()
        .await
        .expect("Unable to create new pool connection");

    let system = sqlx::query!(
        "SELECT constellationID FROM mapSolarSystems WHERE _key IS ?",
        system_id
    )
    .fetch_one(&db_connect)
    .await
    .expect("Unable to query the database");

    Ok(system
        .constellationID
        .expect("Unable to return database record"))

    // let fp = BufReader::new(File::open("./sde/mapSolarSystems.jsonl").await?);
    // let reader = AsyncJsonLinesReader::new(fp);
    // let typesde = reader
    //     .read_all::<SDESystemStruct>()
    //     .try_collect::<Vec<_>>()
    //     .await?;
    // let systemid:i64 = system_id.parse().unwrap(); // Pegasus constellation - home of extremely valuable gas clouds
    // let mut const_id: i64 = 0;
    // for x in typesde {
    //     if x._key == systemid {
    //         const_id = x.constellation_id;
    //     }
    // }
    // Ok(const_id)
}

pub async fn get_planet_number(planet_id: String, client: Client) -> Result<i64, MyError> {
    let db_connect = crate::db_connect().await;
    let pool = db_connect
        .acquire()
        .await
        .expect("Unable to create new pool connection");

    let system = sqlx::query!(
        "SELECT celestialIndex FROM mapPlanets WHERE _key IS ?",
        planet_id
    )
        .fetch_one(&db_connect)
        .await
        .expect("Unable to query the database");

    Ok(system.celestialIndex.expect("Unable to return database record"))
}

pub async fn get_timer_region_id(system_id: String, client: Client) -> Result<i64, MyError> {
    let db_connect = crate::db_connect().await;
    let pool = db_connect
        .acquire()
        .await
        .expect("Unable to create new pool connection");

    let system = sqlx::query!(
        "SELECT regionID FROM mapSolarSystems WHERE _key IS ?",
        system_id
    )
    .fetch_one(&db_connect)
    .await
    .expect("Unable to query the database");

    Ok(system.regionID.expect("Unable to return database record"))

    // let fp = BufReader::new(File::open("./sde/mapSolarSystems.jsonl").await?);
    // let reader = AsyncJsonLinesReader::new(fp);
    // let typesde = reader
    //     .read_all::<SDESystemStruct>()
    //     .try_collect::<Vec<_>>()
    //     .await?;
    // let systemid:i64 = system_id.parse().unwrap(); // Pegasus constellation - home of extremely valuable gas clouds
    // let mut region_id: i64 = 0;
    // for x in typesde {
    //     if x._key == systemid {
    //         region_id = x.region_id.expect("Unable to locate");
    //     }
    // }
    // Ok(region_id)
}

pub async fn get_campaigns() -> Result<Campaigns, reqwest::Error> {
    let url = "https://esi.evetech.net/latest/sovereignty/campaigns/?datasource=tranquility";
    let response = reqwest::get(url).await?;
    let timers: Campaigns = response.json().await?;
    Ok(timers)
}

async fn get_system(system_id: &str) -> Result<SystemInfo, reqwest::Error> {
    let url = format!("https://esi.evetech.net/latest/universe/systems/{system_id}/");
    let response = reqwest::get(url).await?;
    let systeminfo: SystemInfo = response.json().await?;

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

pub async fn get_incursions() -> Result<Incursions, reqwest::Error> {
    let url = "https://esi.evetech.net/latest/incursions/?datasource=tranquility";
    let resp = reqwest::get(url).await?;
    let incursions: Incursions = resp.json().await?;
    Ok(incursions)
}

async fn get_solar_name(system_id: String, client: Client) -> Result<String, MyError> {
    // let db_connect = db_connect().await;
    // let pool = db_connect
    //     .acquire()
    //     .await
    //     .expect("Unable to create new pool connection");
    //
    // let system = sqlx::query!(
    //     "SELECT name_en FROM mapSolarSystems WHERE _key IS ?",
    //     system_id
    // )
    // .fetch_one(&db_connect)
    // .await
    // .expect("Unable to query the database");
    //
    // Ok(system
    //     .solarSystemName
    //     .expect("Unable to return database record"))

    // let fp = BufReader::new(File::open("./sde/mapSolarSystems.jsonl").await?);
    // let reader = AsyncJsonLinesReader::new(fp);
    // let typesde = reader
    //     .read_all::<SDESystemStruct>()
    //     .try_collect::<Vec<_>>()
    //     .await?;
    // let system: i64 = system_id.parse().unwrap();
    // // let mut foundvar = false;
    // let mut name = String::new();
    // for x in typesde {
    //     if x._key == system {
    //         name = x.name.en.expect("couldn't locate itemid in types.jsonl");
    //     }
    // }
    Ok(system_id)
}

pub async fn get_timer_region_name(
    region_id: String,
    client: Client,
) -> Result<String, MyError> {
    let db_connect = crate::db_connect().await;
    let pool = db_connect
        .acquire()
        .await
        .expect("Unable to create new pool connection");

    let system = sqlx::query!(
        "SELECT name_en FROM mapRegions WHERE _key IS ?",
        region_id
    )
    .fetch_one(&db_connect)
    .await
    .expect("Unable to query the database");

    Ok(system.name_en.expect("Unable to return database record"))

    // let fp = BufReader::new(File::open("./sde/mapRegions.jsonl").await?);
    // let reader = AsyncJsonLinesReader::new(fp);
    // let typesde = reader
    //     .read_all::<SDERegionStruct>()
    //     .try_collect::<Vec<_>>()
    //     .await?;
    // let regid:i64 = region_id.parse().unwrap(); // Pegasus constellation - home of extremely valuable gas clouds
    // let mut reg_name = String::new();
    // for x in typesde {
    //     if x._key == regid {
    //         reg_name = x.name.en.expect("couldn't locate itemid in mapRegions.jsonl");
    //     }
    // }
    // Ok(reg_name)
}