use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum Location {
    Station(StationResp),
    Address(AddressResp),
    Poi(PoiResponse),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StationResp {
    // #[serde(rename = "type")]
    // pub type_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub place: String,
    pub name: String,
    pub global_id: String,
    pub diva_id: i32,
    // pub has_live_data: bool,
    pub has_zoom_data: bool,
    pub transport_types: Vec<String>,
    pub surrounding_plan_link: String,
    pub aliases: String,
    pub tariff_zones: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddressResp {
    // #[serde(rename = "type")]
    // pub type_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub place: String,
    pub name: String,
    pub post_code: String,
    pub street: String,
    pub house_number: String,
    // pub poi: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PoiResponse {
    // #[serde(rename = "type")]
    // pub type_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub place: String,
    pub name: String,
}

pub async fn get_station(station_search: &str) -> Result<Vec<Location>, reqwest::Error> {
    let url = format!(
        "https://www.mvg.de/api/fib/v2/location?query={}",
        station_search
    );
    let resp = reqwest::get(url).await?.json::<Vec<Location>>().await?;
    Ok(resp)
}
