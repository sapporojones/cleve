use chrono::Utc;
use crate::MyError;

pub async fn date_calc(date_string: String) -> Result<i64, reqwest::Error> {
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

pub fn date_parse(date_string: &String) -> String {
    let dt: Vec<&str> = date_string.split("T").collect();
    let date = dt[0].replace("\"", "");

    let pfs = chrono::NaiveDate::parse_from_str;

    let naive_dt = pfs(&date, "%Y-%m-%d").expect("unable to parse kill date");
    naive_dt.to_string()
}

pub async fn killmail_time_calc(date_string: String) -> Result<String, MyError> {
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

pub async fn timer_time_calc(date_string: String) -> Result<String, reqwest::Error> {
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