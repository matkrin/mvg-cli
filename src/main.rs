mod colorize;

use anyhow::Result;
use chrono::{Local, NaiveTime, TimeZone};
use clap::{Parser, Subcommand};
use mvg_api::{get_departures, get_notifications, get_routes, get_station, Location};
use nu_ansi_term::Style;
use spinners::{Spinner, Spinners};
use tabled::{
    settings::{object::Columns, Modify, Width},
    Table, Tabled,
};
use terminal_size::{terminal_size, Width as TerminalWidth};

use crate::colorize::colorize_line;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
#[clap(trailing_var_arg = true)]
enum Commands {
    /// Show routes
    #[clap(visible_alias = "r")]
    Routes {
        /// The station from where to go
        from: String,
        /// The station of destination
        to: String,
        /// Specify a time in [HH:MM] for the departure or arrival if -a
        #[arg(short, long)]
        time: Option<String>,
        /// If set, --time specifies the arrival time
        #[arg(short, long, requires = "time")]
        arrival: bool,
    },

    /// Show Departures
    #[clap(visible_alias = "d")]
    Departures {
        /// The station from where depart
        station: String,
        /// Specify a time offset in minutes
        #[arg(short, long)]
        offset: Option<usize>,
    },

    /// Show all notifications or for a specific line
    #[clap(visible_alias = "n")]
    Notifications {
        /// Filter for a specific line
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Show map in browser
    #[clap(visible_alias = "m")]
    Map {
        /// Show the regional map
        #[arg(short, long)]
        region: bool,
        /// Show the tram map
        #[arg(short, long)]
        tram: bool,
        /// Show the map for night lines
        #[arg(short, long)]
        night: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Cli = Cli::parse();

    match args.command {
        Commands::Routes {
            from,
            to,
            time,
            arrival,
        } => {
            handle_routes(from, to, time, arrival).await?;
        }
        Commands::Notifications { filter } => {
            handle_notifications(filter).await?;
        }
        Commands::Departures { station, offset } => {
            handle_departures(station, offset).await?;
        }
        Commands::Map {
            region,
            tram,
            night,
        } => {
            handle_map(region, tram, night)?;
        }
    };

    Ok(())
}

#[derive(Tabled)]
struct RouteTableEntry {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "In")]
    in_minutes: String,
    #[tabled(rename = "Duration")]
    duration: String,
    #[tabled(rename = "Lines")]
    lines: String,
    #[tabled(rename = "Delay")]
    delay: String,
    #[tabled(rename = "Info")]
    info: String,
}

async fn handle_routes(
    from: String,
    to: String,
    time: Option<String>,
    arrival: bool,
) -> Result<()> {
    let mut spinner = Spinner::new(Spinners::Aesthetic, "Fetching...".to_string());
    let from_response = &get_station(&from).await?[0];
    let from_id = match from_response {
        mvg_api::Location::Station(s) => &s.global_id,
        _ => anyhow::bail!("No station {} found", from),
    };
    let to_response = &get_station(&to).await?[0];
    let to_id = match to_response {
        mvg_api::Location::Station(s) => &s.global_id,
        _ => anyhow::bail!("No station {} found", to),
    };
    let time = match time {
        Some(t) => {
            let naive_time = NaiveTime::parse_from_str(&t, "%H:%M")?;
            let naive_datetime = Local::now().date_naive().and_time(naive_time);
            Local.from_local_datetime(&naive_datetime).unwrap()
        }
        None => Local::now(),
    };

    let routes = get_routes(
        from_id,
        to_id,
        Some(time),
        Some(arrival),
        None,
        None,
        None,
        None,
        None,
    )
    .await?;
    let table_entries = routes
        .iter()
        .map(|connection| {
            let origin = &connection.parts[0].from;
            let destination = &connection.parts[connection.parts.len() - 1].to;
            let time = format!(
                "{} - {}",
                origin.planned_departure.format("%H:%M"),
                destination.planned_departure.format("%H:%M")
            );
            let in_minutes = (origin.planned_departure.time() - Local::now().time())
                .num_minutes()
                .to_string();
            let duration = (destination.planned_departure.time() - origin.planned_departure.time())
                .num_minutes()
                .to_string();
            let lines = connection
                .parts
                .iter()
                .map(|x| colorize_line(&x.line.label))
                .collect::<Vec<_>>()
                .join(", ");
            let delay = match origin.departure_delay_in_minutes {
                Some(delay) if delay != 0 => delay.to_string(),
                _ => "-".to_string(),
            };
            let info = connection
                .parts
                .iter()
                .flat_map(|x| x.messages.clone())
                .collect::<Vec<_>>()
                .join("\n");

            RouteTableEntry {
                time,
                in_minutes,
                duration,
                lines,
                delay,
                info,
            }
        })
        .collect::<Vec<_>>();

    let mut table = Table::new(table_entries);
    table.with(tabled::settings::Style::rounded());
    let from_name = match name_from_location(from_response) {
        Some(s) => s,
        None => anyhow::bail!("No station name found for {}", from),
    };
    let to_name = match name_from_location(to_response) {
        Some(s) => s,
        None => anyhow::bail!("No station name found for {}", to),
    };
    spinner.stop_and_persist("✔", format!("Connections for: {} ➜ {}", from_name, to_name));
    println!("{}", table);

    Ok(())
}

#[derive(Tabled)]
struct DeparturesTableEntry {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "In")]
    in_minutes: String,
    #[tabled(rename = "Line")]
    line: String,
    #[tabled(rename = "Destination")]
    destination: String,
    #[tabled(rename = "Delay")]
    delay: String,
    #[tabled(rename = "Info")]
    info: String,
}

async fn handle_departures(station: String, offset: Option<usize>) -> Result<()> {
    let mut spinner = Spinner::new(Spinners::Aesthetic, "Fetching...".to_string());
    let station_response = &get_station(&station).await?[0];
    let station_id = match station_response {
        mvg_api::Location::Station(s) => &s.global_id,
        _ => panic!("No station {} found", station),
    };
    let offset = offset.unwrap_or(0);
    let departures = get_departures(station_id, offset).await?;
    let departures_table_entries = departures.iter().map(|departure| {
        let time = departure.planned_departure_time.format("%H:%M").to_string();
        let in_minutes = (departure.planned_departure_time.time() - Local::now().time())
            .num_minutes()
            .to_string();
        let line = colorize_line(&departure.label);
        let destination = departure.destination.clone();
        let delay = match departure.delay_in_minutes {
            Some(min) if min != 0 => min.to_string(),
            _ => "-".to_string(),
        };
        let info = departure.messages.join("\n");
        DeparturesTableEntry {
            time,
            in_minutes,
            line,
            destination,
            delay,
            info,
        }
    });

    let station_name = match name_from_location(station_response) {
        Some(s) => s,
        None => anyhow::bail!("No station name found for {}", station),
    };

    spinner.stop_and_persist("✔", format!("Departures for: {}", station_name));

    let mut table = Table::new(departures_table_entries);
    table.with(tabled::settings::Style::rounded());
    println!("{}", table);

    Ok(())
}

#[derive(Tabled)]
struct NotificationsTableEntry {
    #[tabled(rename = "Lines")]
    lines: String,
    #[tabled(rename = "Duration")]
    duration: String,
    #[tabled(rename = "Details")]
    details: String,
}

async fn handle_notifications(filter: Option<String>) -> Result<()> {
    let notifications = get_notifications().await?;
    let notifications_table_entries = notifications
        .iter()
        .map(|notification| {
            let lines = notification
                .lines
                .iter()
                .map(|line| colorize_line(&line.name))
                .collect::<Vec<_>>()
                .join(", ");
            let duration_from = notification.active_duration.from_date.format("%d.%m.%Y");
            let duration_to = notification
                .active_duration
                .to_date
                .map(|x| x.format("%d.%m.%Y").to_string())
                .unwrap_or("".to_string());
            let duration = format!("{} - {}", duration_from, duration_to);
            let title = html2text::from_read(notification.title.as_bytes(), 99999);
            let text = html2text::from_read(notification.text.as_bytes(), 99999);
            let details = format!("{}\n{}", Style::new().bold().paint(title), text);
            NotificationsTableEntry {
                lines,
                duration,
                details,
            }
        })
        .collect::<Vec<_>>();

    let notifications_table_entries = match filter {
        Some(f) => notifications_table_entries
            .into_iter()
            .filter(|entry| entry.lines.to_lowercase().contains(&f.to_lowercase()))
            .collect::<Vec<_>>(),
        _ => notifications_table_entries,
    };

    if notifications_table_entries.is_empty() {
        println!("No notifications found");
        return Ok(());
    };

    let (TerminalWidth(terminal_width), _) = terminal_size().expect("Not in a terminal");
    let mut table = Table::new(notifications_table_entries);
    table
        .with(tabled::settings::Style::rounded())
        .with(Modify::new(Columns::first()).with(Width::wrap(10).keep_words()))
        .with(
            Modify::new(Columns::last())
                .with(Width::wrap(terminal_width as usize - 50).keep_words()),
        );

    println!("{}", table);

    Ok(())
}

fn handle_map(region: bool, tram: bool, night: bool) -> Result<()> {
    if let (false, false, false) = (region, tram, night) {
        open::that(
            "https://www.mvg.de/dam/jcr:88249232-e41c-417b-b976-1945c5ade867/netz-tarifplan.pdf",
        )?
    };

    if region {
        open::that(
            "https://www.mvg.de/dam/jcr:88249232-e41c-417b-b976-1945c5ade867/netz-tarifplan.pdf",
        )?;
    }
    if tram {
        open::that("https://www.mvg.de/dam/jcr:1164570c-cc5f-4b6d-a007-e99c32b00905/tramnetz.pdf")?;
    }
    if night {
        open::that(
            "https://www.mvg.de/dam/jcr:fe99cd93-ef1c-483c-a715-f421da96382b/nachtliniennetz.pdf",
        )?;
    }

    Ok(())
}

fn name_from_location(location_response: &Location) -> Option<String> {
    match location_response {
        mvg_api::Location::Station(s) => {
            let a = nu_ansi_term::Style::new().bold().paint(&s.name).to_string();
            let b = nu_ansi_term::Style::new()
                .italic()
                .paint(&s.place)
                .to_string();
            Some([a, b].join(", "))
        }
        _ => None,
    }
}
