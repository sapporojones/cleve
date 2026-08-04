use std::io;
use std::io::Write;
use std::path::PathBuf;
use serde_json::Value;
use reqwest::Client;
use reqwest::header::USER_AGENT;
use zip_extensions::zip_extract;
use futures_util::StreamExt;
use crate::{lookups, AllianceInfo, CcpKillmail, CharInfo, CorpInfo, EveScout, EveStatus, Incursions, MyError, Skyhooks};
use crate::helpers::{date_calc, date_parse};
use crate::lookups::{alliance_info, char_search, corp_info, get_timer_region_id, get_timer_region_name, get_timer_solar_name, get_zkb_stats, item_lookup, kill_resolve, public_info};

pub(crate) async fn shlookup(char_name: &str) -> Result<(), MyError> {
    let client = reqwest::Client::builder()
        .user_agent("Simple zkb stats/kill parser")
        .build()?;
    // // known character ids for testing with:
    // // sappo = 772506501
    // // billy = 1826057122
    // // d1ck = 749151393
    // // comment out below for release version
    // let char_id: &str = "772506501";
    // uncomment below to accept char id from user via command line args
    let ci = char_name;
    let char_id = char_search(ci, client.clone()).await?;
    let p: CharInfo = public_info(char_id.as_str(), client.clone()).await?;

    let corpid: i64 = p.corporation_id;
    let c: CorpInfo = corp_info(corpid.to_string().as_str(), client.clone()).await?;

    let aid = c.alliance_id;

    let zs: Value = get_zkb_stats(char_id.clone(), client.clone()).await?;

    let mut kills_vec = Vec::new();
    let mut losses_vec = Vec::new();
    let mut ships_kills_vec = Vec::new();
    let mut ships_loss_vec = Vec::new();

    // knobs

    let kills_url = format!("https://zkillboard.com/api/kills/characterID/{}/", char_id);

    let kills_response = client.get(kills_url).send().await?;
    let zkb: Value = kills_response.json().await?;

    let zkb_sd: u64 = zs["shipsDestroyed"].as_u64().expect("Expected 'shipsDestroyed'");

    let kill_parse_limit: usize = if zs["shipsDestroyed"]
        .as_i64()
        .expect(zkb_sd.to_string().as_str())
        < i64::from(5)
    {
        let parse_limit = zs["shipsDestroyed"]
            .as_u64()
            .expect("Couldn't determine number of ships interacted with");
        parse_limit as usize
    } else {
        5
    };
    let loss_parse_limit: usize = if zs["shipsLost"]
        .as_i64()
        .expect("Couldn't determine number of ships interacted with")
        < i64::from(5)
    {
        let parse_limit = zs["shipsLost"]
            .as_u64()
            .expect("Couldn't determine number of ships interacted with");
        parse_limit as usize
    } else {
        5
    };

    // let parse_limit = 5;
    let mut current_kill: usize = 0;

    while current_kill < kill_parse_limit {
        let mr_id: String = zkb[current_kill]["killmail_id"].to_string();
        let mr_hash: String = zkb[current_kill]["zkb"]["hash"]
            .to_string()
            .replace("\"", "");

        let mr_kill: CcpKillmail =
            kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;
        kills_vec.push(mr_kill);
        current_kill += 1;
    }
    let mut attack_ship = Vec::new();
    for kill in kills_vec.iter() {
        let killed_with_j: String = item_lookup(
            kill.victim
                .ship_type_id
                .expect("Can't find victim ship type")
                .to_string()
                .replace("\"", ""),
            client.clone(),
        )
            .await?;
        let killed_with: String = killed_with_j.replace("\"", "");
        ships_kills_vec.push(killed_with);

        for attacker in kill.attackers.iter() {
            if attacker.character_id.is_none() {
                // println!("attacker.character_id is none");
                let attack_ship_id = attacker
                    .ship_type_id
                    .expect("Could not determine ship ID")
                    .to_string();
                let attack_tmp = item_lookup(attack_ship_id, client.clone()).await?;
                attack_ship.push(attack_tmp)
            } else if attacker
                .character_id
                .expect("Can't find character ID on kill for some reaosn")
                .to_string()
                == char_id
            {
                if attacker.ship_type_id.is_none() {
                    // println!("attacker.character_id = char_id");
                    let attack_ship_id = attacker
                        .weapon_type_id
                        .expect("Can't find weapon type used")
                        .to_string();
                    let attack_tmp = item_lookup(attack_ship_id, client.clone()).await?;
                    attack_ship.push(attack_tmp)
                } else {
                    // println!("attacker.character_id else condition");
                    let attack_ship_id = attacker
                        .ship_type_id
                        .expect("Could not determine ship ID")
                        .to_string();
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
        let mr_hash: String = zkb[current_loss]["zkb"]["hash"]
            .to_string()
            .replace("\"", "");

        let mr_kill: CcpKillmail =
            kill_resolve(mr_id.to_string(), mr_hash.to_string(), client.clone()).await?;
        losses_vec.push(mr_kill);
        current_loss += 1;
    }
    for loss in losses_vec.iter() {
        let killed_with_j: String = item_lookup(
            loss.victim
                .ship_type_id
                .expect("Can't find victim ship type")
                .to_string()
                .replace("\"", ""),
            client.clone(),
        )
            .await?;
        let lost_with: String = killed_with_j.replace("\"", "");
        ships_loss_vec.push(lost_with);
    }

    println!("\n \nBasic info:");
    println!("Name: {}", p.name.to_string().replace("\"", ""));
    let bday_raw: String = p.birthday.to_string().replace("\"", "");
    let bday_clean: String = date_parse(&bday_raw);

    println!("Birthday: {}", bday_clean);

    let sec_status: String = p.security_status.to_string();
    println!(
        "Security Status: {:}",
        &sec_status.as_str().replace("\"", "")
    );

    println!(
        "\nCorporation: {} [{}]",
        c.name.to_string().replace("\"", ""),
        c.ticker.to_string().replace("\"", "")
    );
    // println!("Ticker: {}", c["ticker"]);
    let tax_rate = c.tax_rate * 100.0;
    println!("Corporation members: {}", c.member_count);
    println!("Corporation tax rate: {}", tax_rate);

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
        }
        Some(_aid) => {
            let alliance_info: AllianceInfo =
                alliance_info(aid.unwrap().to_string(), client.clone()).await?;
            println!(
                "\nAlliance: {} [{}]",
                alliance_info.name.to_string().replace("\"", ""),
                alliance_info.ticker.to_string().replace("\"", "")
            );

            let alliance_bday_raw: String =
                alliance_info.date_founded.to_string().replace("\"", "");
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
        let kill_diff = date_calc(killtime.clone()).await?;
        let killed_with = &ships_kills_vec[idx];
        let kill_system =
            get_timer_solar_name(kill.solar_system_id.to_string(), client.clone()).await?;
        // let kill_const =
        //     get_timer_const_id(kill.solar_system_id.to_string(), client.clone()).await?;
        let kill_region = get_timer_region_id(kill.solar_system_id.to_string(), client.clone()).await?;
        let kill_region_name =
            get_timer_region_name(kill_region.to_string(), client.clone()).await?;
        println!(
            "{} days ago on {} killed a(n) {} while flying a {} in {} - {}",
            kill_diff,
            &killtime_clean,
            killed_with,
            attack_ship[idx],
            kill_system,
            kill_region_name
        );

        idx += 1;
    }

    println!("\nMost recent Losses:");
    let mut idx = 0;
    for loss in losses_vec.iter() {
        let losstime = loss.killmail_time.to_string();
        let losstime_clean: String = date_parse(&losstime);
        let loss_diff = date_calc(losstime.clone()).await?;
        let lost_ship = &ships_loss_vec[idx];
        let loss_system =
            get_timer_solar_name(loss.solar_system_id.to_string(), client.clone()).await?;
        // let loss_const =
        //     get_timer_const_id(loss.solar_system_id.to_string(), client.clone()).await?;
        let loss_region = get_timer_region_id(loss.solar_system_id.to_string(), client.clone()).await?;
        let loss_region_name =
            get_timer_region_name(loss_region.to_string(), client.clone()).await?;
        println!(
            "{} days ago on {} lost a(n) {} in {} - {}",
            loss_diff, &losstime_clean, lost_ship, loss_system, loss_region_name
        );

        idx += 1;
    }

    // Corp history gather/parse/print
    // let mut output: Vec<String> = Vec::new();
    // let corp_history: CorpHistory = get_corp_history(char_id.to_string(), client.clone()).await?;
    // for corp in corp_history {
    //     let corp_name = corp_info(corp.corporation_id.to_string().as_str(), client.clone()).await?;
    //     output.push(format!("{:<32} {:<20}", corp_name.name, corp.start_date))
    // }
    // println!("\nCorp history:\n{:<32}{:<20}", "Corp", "Start Date:");
    // for line in output.iter() {
    //     println!("{}", line);
    // }



    println!("\n \n");
    Ok(())
}

pub(crate) async fn evescout() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let thera_response = client
        .get("https://api.eve-scout.com//v2/public/signatures?system_name=thera")
        .send()
        .await?;
    let thera: EveScout = thera_response.json().await?;
    let turnur_response = client
        .get("https://api.eve-scout.com//v2/public/signatures?system_name=turnur")
        .send()
        .await?;
    let turnur: EveScout = turnur_response.json().await?;

    println!(
        "\nThera\n{:<20} {:<15} {:<15} {:<15} {:<15}",
        "in_region", "in_system", "in_sig", "out_sig", "time_remaining"
    );
    for key in thera.iter() {
        println!(
            "{:<20} {:<15} {:<15} {:<15} {:<15}",
            key.in_region_name,
            key.in_system_name,
            key.in_signature,
            key.out_signature,
            key.remaining_hours
        );
    }
    println!("\n");
    println!(
        "Turnur\n{:<20} {:<15} {:<15} {:<15} {:<15}",
        "in_region", "in_system", "in_sig", "out_sig", "time_remaining"
    );
    for key in turnur.iter() {
        println!(
            "{:<20} {:<15} {:<15} {:<15} {:<15}",
            key.in_region_name,
            key.in_system_name,
            key.in_signature,
            key.out_signature,
            key.remaining_hours
        );
    }
    println!("\n");
    Ok(())
}

pub(crate) async fn thera() -> Result<(), reqwest::Error> {
    let thera_response =
        reqwest::get("https://api.eve-scout.com//v2/public/signatures?system_name=thera").await?;
    let thera: EveScout = thera_response.json().await?;
    println!(
        "\nThera\n{:<20} {:<15} {:<15} {:<15} {:<15}",
        "in_region", "in_system", "in_sig", "out_sig", "time_remaining"
    );
    for key in thera.iter() {
        println!(
            "{:<20} {:<15} {:<15} {:<15} {:<15}",
            key.in_region_name,
            key.in_system_name,
            key.in_signature,
            key.out_signature,
            key.remaining_hours
        );
    }
    println!("\n");
    Ok(())
}

pub(crate) async fn turnur() -> Result<(), reqwest::Error> {
    let turnur_response =
        reqwest::get("https://api.eve-scout.com//v2/public/signatures?system_name=turnur").await?;
    let turnur: EveScout = turnur_response.json().await?;
    println!(
        "\nTurnur\n{:<20} {:<15} {:<15} {:<15} {:<15}",
        "in_region", "in_system", "in_sig", "out_sig", "time_remaining"
    );
    for key in turnur.iter() {
        println!(
            "{:<20} {:<15} {:<15} {:<15} {:<15}",
            key.in_region_name,
            key.in_system_name,
            key.in_signature,
            key.out_signature,
            key.remaining_hours
        );
    }
    println!("\n");

    Ok(())
}

pub(crate) async fn status() -> Result<(), reqwest::Error> {
    let url = "https://esi.evetech.net/latest/status/?datasource=tranquility";
    let status_response = reqwest::get(url).await?;
    let status: EveStatus = status_response.json().await?;
    println!("\nPlayers online: {}", status.players.to_string().as_str());
    println!(
        "Current server version: {}",
        status.server_version.to_string().as_str()
    );
    println!(
        "Server start time: {}\n",
        status.start_time.to_string().as_str()
    );

    Ok(())
}

pub async fn get_sde_components() -> Result<(), Box<dyn std::error::Error>> {


    // let mut sde_present: bool = true;

    // println!("Checking for previous SDE and removing if found...");

    // sde_present = Path::new("sqlite-latest.sqlite").exists();
    // if sde_present {
    //     remove_file("sqlite-latest.sqlite").unwrap();
    // }

    let client = Client::new();

    println!("Downloading latest SDE...");

    // let mut resp_sde = client
    //     .get("https://developers.eveonline.com/static-data/eve-online-static-data-latest-jsonl.zip")
    //     .header(USER_AGENT, "CLI EVE Utility Application by Sapporo Jones")
    //     .send()
    //     .await?
    //     .bytes_stream();

    let mut resp_sde = client
        .get("https://chat.sunkenrlyeh.com/limited_sde.zip")
        .header(USER_AGENT, "CLI EVE Utility Application by Sapporo Jones")
        .send()
        .await?
        .bytes_stream();

    let mut out_sde = tokio::fs::File::create("eve-online-static-data-latest-jsonl.zip")
        .await
        .expect("failed to create file");

    while let Some(item) = resp_sde.next().await {
        tokio::io::copy(&mut item?.as_ref(), &mut out_sde)
            .await
            .expect("Unable to write data");
    }

    println!("Decompressing SDE...");

    let sde_path = r"./eve-online-static-data-latest-jsonl.zip";
    let archive_path: PathBuf = PathBuf::from(r#"eve-online-static-data-latest-jsonl.zip"#);
    let target_path: PathBuf = PathBuf::from(r#"."#);
    zip_extract(&archive_path, &target_path)?;

    // let sde_compressed_file = std::fs::File::open("sqlite-latest.sqlite.bz2");
    // let sde_decompressed_output = std::fs::File::create("sqlite-latest.sqlite");
    // let mut sde_reader = DecoderReader::new(sde_compressed_file?);
    // std::io::copy(&mut sde_reader, &mut sde_decompressed_output.unwrap())
    //     .expect("Unable to write contents of SDE");



    println!("Done!");

    Ok(())
}

pub async fn incursions() -> Result<(), MyError> {
    let client = reqwest::Client::new();
    let mut output = Vec::new();
    let incursions: Incursions = crate::lookups::get_incursions().await?;

    for incursion in incursions.iter() {
        // let const_info = get_const(incursion.constellation_id.to_string().as_str()).await?;
        let region_id =
            get_timer_region_id(incursion.staging_solar_system_id.to_string(), client.clone()).await?;
        let region_name = get_timer_region_name(region_id.to_string(), client.clone()).await?;
        let staging_system = get_timer_solar_name(
            incursion.staging_solar_system_id.to_string(),
            client.clone(),
        )
        .await?;
        let state = incursion.state.as_str();
        let has_boss = incursion.has_boss;
        let out_string = format!(
            "{:<30} {:<20} {:<20} {:<20}",
            region_name, staging_system, state, has_boss
        );
        output.push(out_string)
    }
    println!(
        "\n{:<30} {:<20} {:<20} {:<20}",
        "Region:", "Staging System:", "State:", "Has Boss:"
    );
    for incursion in output.iter() {
        println!("{incursion}")
    }
    println!("\n");

    Ok(())
}

pub async fn timers() -> Result<(), MyError> {
    let client = reqwest::Client::new();

    let current_timers = crate::lookups::get_campaigns().await?;
    let total_timers = current_timers.len();

    let mut output: Vec<String> = Vec::new();
    print!("Processing {total_timers} timers... ");
    io::stdout().flush().unwrap();


    for timer in current_timers.iter() {


        let system_name =
            get_timer_solar_name(timer.solar_system_id.to_string(), client.clone()).await?;


        let region_id =
            get_timer_region_id(timer.solar_system_id.to_string(), client.clone()).await?;
        let region_name = get_timer_region_name(region_id.to_string(), client.clone()).await?;


        let defender_value = lookups::alliance_info(timer.defender_id.to_string(), client.clone()).await?;
        let defender = defender_value.name;

        let timer_start = crate::helpers::timer_time_calc(timer.start_time.to_string()).await?;



        output.push(format!(
            "{:<20} {:<20} {:<20} {:<50} {:<20}",
            timer_start,
            region_name,
            system_name,
            defender.replace("\"", ""),
            timer.attackers_score.to_string().as_str()
        ));
        print!("*");
        io::stdout().flush().unwrap();
    }

    println!(
        "\n{:<20} {:<20} {:<20} {:<50} {:<20}",
        "timer start:", "region:", "solar_system:", "defender:", "attacker_score:"
    );
    for line in output.iter() {
        println!("{}", line);
        io::stdout().flush().unwrap();
    }
    println!("\n");
    Ok(())
}

pub async fn system_stats(sys_name: &str) -> Result<(), MyError> {
    let client = reqwest::Client::builder()
        .user_agent("A simple zkb stats/kills parser")
        .build()?;
    let system_name = sys_name.to_uppercase();
    let system_id_lookup = crate::lookups::name_lookup(system_name.to_string(), client.clone()).await?;

    println!("Looking up system name...");
    let system_id: String = system_id_lookup["systems"][0]["id"].to_string();
    println!("Looking up system id...");
    let system_zkb = crate::lookups::get_system_kills(system_id.as_str(), client.clone()).await?;
    println!("Retrieving zkillboard for {sys_name}...");
    let kills = crate::lookups::get_num_kills(system_id.as_str(), client.clone()).await?;
    println!("Retrieving total number of ships killed in system in the last hour...");

    println!("Retrieving total number of NPCs killed in system in the last hour...");
    let system_jumps = crate::lookups::get_jumps(system_id.as_str(), client.clone()).await?;
    println!("Retrieving total number of jumps in system in the last hour...");
    let system_gates = crate::lookups::get_gates(system_id.as_str(), client.clone()).await?;
    println!("Determining number of available stargates...");
    let mut ccp_kills: Vec<CcpKillmail> = Vec::with_capacity(5);

    let mut kill_counter: i32 = 0;
    println!("Resolving most recent kills in system...");
    for key in system_zkb.iter() {
        let k = lookups::kill_resolve(
            key.killmail_id.to_string(),
            key.zkb.hash.to_string(),
            client.clone(),
        )
        .await?;
        ccp_kills.push(k);

        kill_counter += 1;
        if kill_counter == 5 {
            break;
        }
    }

    // let const_id = get_timer_const_id(system_id.clone(), client.clone()).await?;
    let region_id = get_timer_region_id(system_id.clone().to_string(), client.clone()).await?;
    let region_name = get_timer_region_name(region_id.to_string(), client.clone()).await?;

    let ship = String::new();
    let mut char = String::new();
    let corp = String::new();
    let alli = String::new();

    let mut outputwrapper = Vec::new();
    let alli = String::new();

    // container values for max len checking later
    let mut kdlen: u8 = 0;
    let mut stlen: u8 = 0;
    let mut charlen: u8 = 0;
    let mut corplen: u8 = 0;
    let mut alllen: u8 = 0;

    for kill in ccp_kills {
        let mut output: Vec<String> = Vec::new();
        let ship = item_lookup(
            kill.victim
                .ship_type_id
                .expect("Can't find victim ship type")
                .to_string(),
            client.clone(),
        )
        .await?;
        if kill.victim.character_id.is_none() {
            char = "None".to_string()
        } else {
            let resp: Value = crate::lookups::legacy_item_lookup(
                kill.victim.character_id.unwrap().to_string(),
                client.clone(),
            )
            .await?;

            char = resp[0]["name"].to_string()
        }
        let corp = lookups::corp_info(
            kill.victim.corporation_id.unwrap().to_string().as_str(),
            client.clone(),
        )
        .await?;
        let mut alli = String::new();

        match kill.victim.alliance_id {
            Some(i) => {
                let x = lookups::alliance_info(kill.victim.alliance_id.unwrap().to_string(), client.clone())
                    .await?;
                alli = x.name
            }
            None => {
                "None";
            }
        }

        let killdelta = crate::helpers::killmail_time_calc(kill.killmail_time).await?;

        output.push(killdelta);

        output.push(ship.to_string());
        output.push(char);
        output.push(corp.name);
        output.push(alli);
        outputwrapper.push(output);
    }

    // Need to determine the longest value in the output vecs in order to size columns correctly



    println!(
        "\nMost recent kill info for {system_name}:\n{:<15} {:<30} {:<25} {:<37} {:<25}",
        "Kill Age:", "Victim Ship:", "Victim Name:", "Victim Corp:", "Victim Alliance:"
    );

    for kill in outputwrapper {
        println!(
            "{:<15} {:<30} {:<25} {:<37} {:<25}",
            kill.first().unwrap().as_str().replace("\"", ""),
            kill.get(1).unwrap().as_str().replace("\"", ""),
            kill.get(2).unwrap().as_str().replace("\"", ""),
            kill.get(3).unwrap().as_str().replace("\"", ""),
            kill.get(4).unwrap().as_str().replace("\"", "")
        );
    }
    let npckills = String::new();
    let podkills = String::new();
    let shipkills = String::new();

    let npckills = kills.first().unwrap().to_string();
    let podkills = kills.get(1).unwrap().to_string();
    let shipkills = kills.get(2).unwrap().to_string();

    println!(
        "\nDotlan Map URL: https://evemaps.dotlan.net/map/{}/{}",
        region_name.replace(" ", "_"),
        system_name
    );

    println!("\nShips destroyed last hour: \t{:<30}", shipkills);
    println!("Capsules destroyed last hour: \t{:<30}", podkills);
    println!("NPCs destroyed last hour: \t{:<30}", npckills);
    println!("Jumps last hour: \t\t{:<30}", system_jumps);
    println!("Number of stargates in system: \t{:<30}\n", system_gates);
    Ok(())
}

pub async fn get_upcoming_skyhooks() -> Result<(), MyError> {
    let client = reqwest::Client::builder()
        .user_agent("A simple zkb stats/kills parser")
        .build()?;
    let db_connect = crate::db_connect().await;
    let pool = db_connect
        .acquire()
        .await
        .expect("Unable to create new pool connection");

    // let url = format!("https://zkillboard.com/api/stats/characterID/{}/", char_id);
    let url = "https://esi.evetech.net/skyhooks/raidable";

    let response = client.get(url)
        .header("X-Compatibility-Date", "2026-05-19")
        .send()
        .await?;


    let skyhooks: Skyhooks = response.json().await?;

    let mut output: Vec<String> = Vec::new();
    let total_timers = skyhooks.skyhooks.len();
    print!("Processing {total_timers} skyhook timers... ");
    io::stdout().flush().unwrap();

    let mut hook_reg = String::from("");
    let mut hook_reg_id = String::from("");
    let mut hook_sys = String::from("");
    let mut hook_planets = String::from("");
    let mut hook_start = String::from("");
    let mut hook_end = String::from("");

    for hook in skyhooks.skyhooks.iter() {
        let hook_sys = lookups::get_timer_solar_name(hook.solar_system_id.to_string(), client.clone()).await?;
        let hook_reg_id = lookups::get_timer_region_id(hook.solar_system_id.to_string(), client.clone()).await?;
        let hook_reg = get_timer_region_name(hook_reg_id.to_string(), client.clone()).await?;
        let hook_planets = lookups::get_planet_number(hook.planet_id.to_string(), client.clone()).await?;
        let hook_start = hook.theft_vulnerability.start.clone();
        let hook_end = hook.theft_vulnerability.end.clone();
        output.push(format!("{:<35} {:<9} {:<7} {:<25} {:<25}", hook_reg, hook_sys, hook_planets, hook_start, hook_end));
        print!("*");
        io::stdout().flush().unwrap();
    }

    println!(
        "\nUpcoming Skyhook Vuln Timers:\n{:<35} {:<9} {:7} {:<25} {:<25}",
        "Region:",
        "System:",
        "Planet:",
        "Start Time:",
        "End Time:"
    );
    for line in output.iter() {
        println!("{}", line);
    }
    Ok(())
}

pub async fn get_hooks_by_region(region_name: String) -> Result<(), MyError> {
    let client = reqwest::Client::builder()
        .user_agent("A simple zkb stats/kills parser")
        .build()?;
    let db_connect = crate::db_connect().await;
    let pool = db_connect
        .acquire()
        .await
        .expect("Unable to create new pool connection");

    // let url = format!("https://zkillboard.com/api/stats/characterID/{}/", char_id);
    let url = "https://esi.evetech.net/skyhooks/raidable";

    let response = client.get(url)
        .header("X-Compatibility-Date", "2026-05-19")
        .send()
        .await?;



    let skyhooks: Skyhooks = response.json().await?;

    let mut output: Vec<String> = Vec::new();
    let total_timers = skyhooks.skyhooks.len();
    print!("Processing {total_timers} skyhook timers... ");
    io::stdout().flush().unwrap();

    let mut hook_reg = String::from("");
    let mut hook_reg_id = String::from("");
    let mut hook_sys = String::from("");
    let mut hook_planets = String::from("");
    let mut hook_start = String::from("");
    let mut hook_end = String::from("");

    for hook in skyhooks.skyhooks.iter() {
        let hook_sys = lookups::get_timer_solar_name(hook.solar_system_id.to_string(), client.clone()).await?;
        let hook_reg_id = lookups::get_timer_region_id(hook.solar_system_id.to_string(), client.clone()).await?;
        let hook_reg = get_timer_region_name(hook_reg_id.to_string(), client.clone()).await?;
        let hook_planets = lookups::get_planet_number(hook.planet_id.to_string(), client.clone()).await?;
        let hook_start = hook.theft_vulnerability.start.clone();
        let hook_end = hook.theft_vulnerability.end.clone();
        if hook_reg.to_uppercase() == region_name.to_uppercase() {
            output.push(format!("{:<9} {:<7} {:<25} {:<25}", hook_sys, hook_planets, hook_start, hook_end));
        }
        print!("*");
        io::stdout().flush().unwrap();

    }
    let local_timers = output.len();
    println!("\n{} timer(s) found for the region of {}", local_timers, region_name);
    println!(
        "\n{:<9} {:7} {:<25} {:<25}",
        "System:",
        "Planet:",
        "Start Time:",
        "End Time:"
    );
    for line in output.iter() {
        println!("{}", line);
    }



    Ok(())
}