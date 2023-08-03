use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use mvg_api::{get_departures, get_notifications, get_routes, get_station};
use tabled::{
    settings::{object::Columns, Modify, Width},
    Table, Tabled,
};
use terminal_size::{terminal_size, Width as TerminalWidth};

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
    println!("{:?}", args);

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
    println!(
        "routes with {:?}, {:?}, {:?}, {:?}",
        from, to, time, arrival
    );
    let from = &get_station(&from).await?[0];
    let from_id = match from {
        mvg_api::Location::Station(s) => &s.global_id,
        _ => todo!(),
    };
    let to = &get_station(&to).await?[0];
    let to_id = match to {
        mvg_api::Location::Station(s) => &s.global_id,
        _ => todo!(),
    };
    let routes = get_routes(from_id, to_id, None, None, None, None, None, None, None).await?;
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
            let lines = connection.parts.iter().fold(Vec::new(), |mut acc, x| {
                acc.push(x.line.label.clone());
                acc
            });
            let lines = lines
                .iter()
                .map(|x| colorize_line(x))
                .collect::<Vec<_>>()
                .join(", ");
            let delay = match origin.departure_delay_in_minutes {
                Some(delay) if delay != 0 => delay.to_string(),
                _ => "-".to_string(),
            };
            let info = connection
                .parts
                .iter()
                .fold(Vec::new(), |mut acc, x| {
                    for message in &x.messages {
                        acc.push(message.clone());
                    }
                    acc
                })
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
    println!("departures with {:?}, {:?}", station, offset);
    let station = &get_station(&station).await?[0];
    let station_id = match station {
        mvg_api::Location::Station(s) => &s.global_id,
        _ => todo!(),
    };
    let departures = get_departures(station_id).await?;
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
        let info = departure
            .messages
            .iter()
            .fold(Vec::new(), |mut acc, x| {
                acc.push(x.clone());
                acc
            })
            .join("\n");
        DeparturesTableEntry {
            time,
            in_minutes,
            line,
            destination,
            delay,
            info,
        }
    });

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
    use nu_ansi_term::Style;
    println!("notifications with {:?}", filter);
    let notifications = get_notifications().await?;
    // dbg!(&notifications[0]);
    let notifications_table_entries = notifications
        .iter()
        .map(|notification| {
            let lines = notification
                .lines
                .iter()
                .map(|line| line.name.clone())
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
        return Ok(())
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

fn colorize_line(line: &str) -> String {
    if line.starts_with('U') {
        colorized_ubahn(line)
    } else if line.starts_with('S') {
        colorize_sbahn(line)
    } else {
        line.to_string()
    }
}

fn colorized_ubahn(line: &str) -> String {
    use nu_ansi_term::Color::Fixed;
    use nu_ansi_term::Style;
    let colored = match line {
        "U1" => Fixed(255).on(Fixed(22)).paint(format!(" {} ", line)),
        "U2" => Fixed(255).on(Fixed(124)).paint(format!(" {} ", line)),
        "U3" => Fixed(255).on(Fixed(166)).paint(format!(" {} ", line)),
        "U4" => Fixed(255).on(Fixed(30)).paint(format!(" {} ", line)),
        "U5" => Fixed(255).on(Fixed(94)).paint(format!(" {} ", line)),
        "U6" => Fixed(255).on(Fixed(20)).paint(format!(" {} ", line)),
        "U7" => {
            let mut i = line.chars();
            let lhs = i.next().unwrap();
            let rhs = i.next().unwrap();
            let lhs = Fixed(255).on(Fixed(22)).paint(format!(" {}", lhs));
            let rhs = Fixed(255).on(Fixed(124)).paint(format!("{} ", rhs));
            let total = [lhs.to_string(), rhs.to_string()].join("");
            Style::new().paint(total)
        }
        "U8" => {
            let mut i = line.chars();
            let lhs = i.next().unwrap();
            let rhs = i.next().unwrap();
            let lhs = Fixed(255).on(Fixed(124)).paint(format!(" {}", lhs));
            let rhs = Fixed(255).on(Fixed(166)).paint(format!("{} ", rhs));
            let total = [lhs.to_string(), rhs.to_string()].join("");
            Style::new().paint(total)
        }
        _ => Style::default().paint(line),
    };
    colored.to_string()
}

fn colorize_sbahn(line: &str) -> String {
    use nu_ansi_term::Color::Fixed;
    use nu_ansi_term::Style;
    let colored = match line {
        "S1" => Fixed(255).on(Fixed(73)).paint(format!(" {} ", line)),
        "S2" => Fixed(255).on(Fixed(34)).paint(format!(" {} ", line)),
        "S3" => Fixed(255).on(Fixed(53)).paint(format!(" {} ", line)),
        "S4" => Fixed(255).on(Fixed(196)).paint(format!(" {} ", line)),
        "S6" => Fixed(255).on(Fixed(29)).paint(format!(" {} ", line)),
        "S7" => Fixed(255).on(Fixed(204)).paint(format!(" {} ", line)),
        "S8" => {
            let mut i = line.chars();
            let lhs = i.next().unwrap();
            let rhs = i.next().unwrap();
            let lhs = Fixed(255).on(Fixed(233)).paint(format!(" {}", lhs));
            let rhs = Fixed(255).on(Fixed(226)).paint(format!("{} ", rhs));
            let total = [lhs.to_string(), rhs.to_string()].join("");
            Style::new().paint(total)
        }
        "S20" => Fixed(255).on(Fixed(203)).paint(format!(" {} ", line)),
        _ => Style::default().paint(line),
    };
    colored.to_string()
}
