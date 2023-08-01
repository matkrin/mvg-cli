use anyhow::Result;
use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use tabled::{Table, Tabled};
use mvg_api::{get_routes, get_station};

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

    /// Show notifications for specific lines or all notifications if no arguments are given
    #[clap(visible_alias = "n")]
    Notifications {
        /// Filter for specific lines
        #[arg(short, long, num_args = 1..)]
        filter: Option<Vec<String>>,
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
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Cli = Cli::parse();
    println!("{:?}", args);

    match args.command {
        Commands::Routes { from, to, time, arrival, } => { handle_routes(from, to, time, arrival).await?; },
        Commands::Notifications { filter } => { handle_notifications(filter)?; },
        Commands::Departures { station, offset } => { handle_departures(station, offset)?; }, 
        Commands::Map { region, tram, night } => { handle_map(region, tram, night)?; },
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

async fn handle_routes(from: String, to: String, time: Option<String>, arrival: bool) -> Result<()> {
    println!("routes with {:?}, {:?}, {:?}, {:?}", from, to, time, arrival);
    let from = &get_station(&from).await?[0];
    let from_id = match from {
        mvg_api::Location::Station(s) => &s.global_id,
        _ => todo!(),
    };
    let to = &get_station(&to).await?[0];
    let to_id = match to {
        mvg_api::Location::Station(s) => &s.global_id,
        _ => todo!()
    };
    let routes = get_routes(from_id, to_id, None , None, None, None, None, None, None).await?;
    // dbg!(&routes);
    let table_entries = routes.iter().map(|connection| {
        let origin = &connection.parts[0].from;
        let destination = &connection.parts[connection.parts.len()-1].to;
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
        let lines = connection.parts.iter().fold(Vec::new(), |mut acc, x| {
            acc.push(x.line.label.clone());
            acc
        }).to_owned().join(", ");
        let delay = match origin.departure_delay_in_minutes {
            Some(delay) if delay != 0 => delay.to_string(),
            _ => "-".to_string(),
        };
        let info = connection.parts.iter().fold(Vec::new(), |mut acc, x| {
            for message in &x.messages {
                acc.push(message.clone());
            }
            acc
        }).join("\n");

        RouteTableEntry { time, in_minutes, duration, lines, delay, info }
    }).collect::<Vec<_>>();
    let mut table = Table::new(table_entries);
    table.with(tabled::settings::Style::rounded());
    println!("{}", table);

    Ok(())
}

fn handle_notifications(filter: Option<Vec<String>>) -> Result<()> {
    println!("notifications with {:?}", filter);
    Ok(())
}

fn handle_departures(station: String, offset: Option<usize>) -> Result<()> {
    println!("departures with {:?}, {:?}", station, offset);
    Ok(())
}

fn handle_map(region: bool, tram: bool, night: bool) -> Result<()> {
    if let (false, false, false) = (region, tram, night) {
        open::that("https://www.mvg.de/dam/jcr:88249232-e41c-417b-b976-1945c5ade867/netz-tarifplan.pdf")?
    };

    if region {
        open::that("https://www.mvg.de/dam/jcr:88249232-e41c-417b-b976-1945c5ade867/netz-tarifplan.pdf")?;
    }
    if tram {
        open::that("https://www.mvg.de/dam/jcr:1164570c-cc5f-4b6d-a007-e99c32b00905/tramnetz.pdf")?;
    }
    if night {
        open::that("https://www.mvg.de/dam/jcr:fe99cd93-ef1c-483c-a715-f421da96382b/nachtliniennetz.pdf")?;
    }

    Ok(())
}
