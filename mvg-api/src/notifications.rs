use chrono::{DateTime, Local};
use serde::Deserialize;
use serde_with::TimestampMilliSeconds;

#[serde_with::serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub title: String,
    pub description: String, // The HTML
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub publication: DateTime<Local>,
    pub publication_duration: Duration,
    pub incident_durations: Vec<Duration>,
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub valid_from: DateTime<Local>,
    #[serde_as(as = "Option<TimestampMilliSeconds<i64>>")]
    pub valid_to: Option<DateTime<Local>>,
    #[serde(rename = "type")]
    pub type_name: String,
    pub provider: String,
    pub links: Vec<NotificationLink>,
    pub lines: Vec<NotificationLines>,
    pub station_global_ids: Vec<String>,
    pub event_types: Vec<String>,
}

#[serde_with::serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Duration {
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    pub from: DateTime<Local>,
    #[serde_as(as = "Option<TimestampMilliSeconds<i64>>")]
    pub to: Option<DateTime<Local>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NotificationLines {
    pub label: String,
    pub transport_type: String,
    pub network: String,
    pub diva_id: String,
    pub sev: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStation {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NotificationLink {
    pub text: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadLink {
    pub id: String,
    pub name: String,
    pub mime_type: String,
}

pub async fn get_notifications() -> Result<Vec<Notification>, reqwest::Error> {
    //let url = "https://www.mvg.de/api/ems/tickers".to_string();
    let url = "https://www.mvg.de/api/bgw-pt/v3/messages".to_string();
    let resp = reqwest::get(url).await?.json::<Vec<Notification>>().await?;
    Ok(resp)
}
