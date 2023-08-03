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
    pub place: String,
    pub name: String,
    // #[serde_as(as = "Rfc3339")]
    pub planned_departure: DateTime<Local>,
    pub departure_delay_in_minutes: Option<isize>,
    pub arrival_delay_in_minutes: Option<isize>,
    pub transport_types: Vec<String>,
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
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PathDescription {
    pub from_path_coord_idx: isize,
    pub to_path_coord_idx: isize,
    pub level: isize,
}

pub async fn get_routes(
    from_station_id: &str,
    to_station_id: &str,
    time: Option<DateTime<Local>>,
    arrival: Option<bool>,
    include_ubahn: Option<bool>,
    include_bus: Option<bool>,
    include_tram: Option<bool>,
    include_sbahn: Option<bool>,
    include_taxi: Option<bool>,
) -> Result<Vec<Connection>, reqwest::Error> {
    let mut transport_types = Vec::new();

    if let Some(x) = include_ubahn {
        if x {
            transport_types.push("UBAHN")
        }
    }
    if let Some(x) = include_bus {
        if x {
            transport_types.push("BUS")
        }
    }
    if let Some(x) = include_tram {
        if x {
            transport_types.push("TRAM")
        }
    }
    if let Some(x) = include_sbahn {
        if x {
            transport_types.push("SBAHN")
        }
    }
    if let Some(x) = include_taxi {
        if x {
            transport_types.push("RUFTAXI")
        }
    }

    let time: DateTime<Utc> = match time {
        Some(t) => DateTime::from(t),
        None => Utc::now(),
    };

    let url = format!(
        "https://www.mvg.de/api/fib/v2/connection?originStationGlobalId={}&destinationStationGlobalId={}&routingDateTime={}&routingDateTimeIsArrival={}&transportTypes={}",
        from_station_id,
        to_station_id,
        time.to_rfc3339_opts(SecondsFormat::Millis, true),
        arrival.unwrap_or(false),
        transport_types.join(","),
    );

    let resp = reqwest::get(url).await?.json::<Vec<Connection>>().await?;
    Ok(resp)
}
