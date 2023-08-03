use chrono::{DateTime, Local};
use serde::Deserialize;
use serde_with::TimestampMilliSeconds;

#[serde_with::serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Departure {
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub planned_departure_time: DateTime<Local>,
    pub realtime: bool,
    pub delay_in_minutes: Option<isize>,
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub realtime_departure_time: DateTime<Local>,
    pub transport_type: String,
    pub label: String,
    pub network: String,
    pub train_type: String,
    pub destination: String,
    pub cancelled: bool,
    pub sev: bool,
    pub platform: Option<usize>,
    pub stop_position_number: Option<usize>,
    pub messages: Vec<String>,
    pub banner_hash: String,
    pub occupancy: String,
    pub stop_point_global_id: String,
}

pub async fn get_departures(station_id: &str, offset_in_min: usize) -> Result<Vec<Departure>, reqwest::Error> {
    let url = format!("https://www.mvg.de/api/fib/v2/departure?globalId={}&limit=10&offsetInMinutes={}&transportTypes=UBAHN,TRAM,BUS,SBAHN,SCHIFF", station_id, offset_in_min);
    let resp = reqwest::get(url).await?.json::<Vec<Departure>>().await?;
    Ok(resp)
}
