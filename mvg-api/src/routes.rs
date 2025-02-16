use chrono::{DateTime, Local, SecondsFormat, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub unique_id: isize,
    pub parts: Vec<ConnectionPart>,
    pub ticketing_information: TicketingInformation,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionPart {
    pub from: Station,
    pub to: Station,
    pub intermediate_stops: Vec<Station>,
    pub no_change_required: bool,
    pub line: Line,
    pub path_polyline: String,
    pub interchange_path_polyline: String,
    pub path_description: Vec<PathDescription>,
    pub exit_letter: String,
    pub distance: f64,
    pub occupancy: String,
    pub messages: Vec<String>,
    pub infos: Vec<String>,
    pub real_time: bool,
}

// #[serde_with::serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    pub latitude: f64,
    pub longitude: f64,
    pub station_global_id: String,
    pub station_diva_id: usize,
    pub platform: Option<usize>,
    pub platfrom_changed: Option<bool>,
    pub place: String,
    pub name: String,
    // #[serde_as(as = "Rfc3339")]
    pub planned_departure: DateTime<Local>,
    pub departure_delay_in_minutes: Option<isize>,
    pub arrival_delay_in_minutes: Option<isize>,
    pub transport_types: Vec<String>,
    pub is_via_stop: Option<bool>,
    pub surrounding_plan_link: String,
    pub occupancy: String,
    pub has_zoom_data: bool,
    pub has_out_of_order_escalator: bool,
    pub has_out_of_order_elevator: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub label: String,
    pub transport_type: String,
    pub destination: String,
    pub train_type: String,
    pub network: String,
    pub sev: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TicketingInformation {
    pub zones: Vec<usize>,
    pub alternative_zones: Vec<usize>,
    pub unified_ticket_ids: Vec<String>,
    pub distance: Option<f64>,
    pub banner_hash: Option<String>,
    pub refresh_id: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PathDescription {
    pub from_path_coord_idx: isize,
    pub to_path_coord_idx: isize,
    pub level: isize,
}

pub struct GetRoutesConfig {
    include_ubahn: bool,
    include_bus: bool,
    include_tram: bool,
    include_sbahn: bool,
    include_taxi: bool,
}

impl Default for GetRoutesConfig {
    fn default() -> Self {
        Self {
            include_ubahn: true,
            include_bus: true,
            include_tram: true,
            include_sbahn: true,
            include_taxi: true,
        }
    }
}

pub async fn get_routes(
    from_station_id: &str,
    to_station_id: &str,
    time: Option<DateTime<Local>>,
    arrival: Option<bool>,
    get_routes_config: GetRoutesConfig,
) -> Result<Vec<Connection>, reqwest::Error> {
    let mut transport_types = Vec::new();

    if get_routes_config.include_ubahn {
        transport_types.push("UBAHN")
    }
    if get_routes_config.include_bus {
        transport_types.push("BUS")
    }
    if get_routes_config.include_tram {
        transport_types.push("TRAM")
    }
    if get_routes_config.include_sbahn {
        transport_types.push("SBAHN")
    }
    if get_routes_config.include_taxi {
        transport_types.push("RUFTAXI")
    }

    let time: DateTime<Utc> = match time {
        Some(t) => DateTime::from(t),
        None => Utc::now(),
    };

    let url = format!(
        //"https://www.mvg.de/api/fib/v2/connection?originStationGlobalId={}&destinationStationGlobalId={}&routingDateTime={}&routingDateTimeIsArrival={}&transportTypes={}",
        "https://www.mvg.de/api/bgw-pt/v3/routes?originStationGlobalId={}&destinationStationGlobalId={}&routingDateTime={}&routingDateTimeIsArrival={}&transportTypes={}",
        from_station_id,
        to_station_id,
        time.to_rfc3339_opts(SecondsFormat::Millis, true),
        arrival.unwrap_or(false),
        transport_types.join(","),
    );

    let resp = reqwest::get(url).await?.json::<Vec<Connection>>().await?;
    Ok(resp)
}
