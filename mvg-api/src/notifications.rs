use chrono::{DateTime, Local};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub title: String,
    pub text: String,
    pub html_text: String,
    pub lines: Vec<NotificationLines>,
    pub incidents: Vec<String>,
    pub links: Vec<NotificationLink>,
    pub download_links: Vec<DownloadLink>,
    pub incident_duration: Vec<Duration>,
    pub active_duration: Duration,
    pub modification_date: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NotificationLines {
    pub id: String,
    pub name: String,
    pub type_of_transport: String,
    pub stations: Vec<NotificationStation>,
    pub direction: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStation {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Duration {
    pub from_date: DateTime<Local>,
    pub to_date: Option<DateTime<Local>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NotificationLink {
    pub href: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadLink {
    pub id: String,
    pub name: String,
    pub mime_type: String,
}

pub async fn get_notifications() -> Result<Vec<Notification>, reqwest::Error> {
    let url = "https://www.mvg.de/api/ems/tickers".to_string();
    let resp = reqwest::get(url).await?.json::<Vec<Notification>>().await?;
    Ok(resp)
}
