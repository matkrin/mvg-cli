use nu_ansi_term::{Color::Fixed, Style};

pub fn colorize_line(line: &str) -> String {
    if line.starts_with('U') {
        colorized_ubahn(line)
    } else if line.starts_with('S') {
        colorize_sbahn(line)
    } else {
        line.to_string()
    }
}

fn colorized_ubahn(line: &str) -> String {
    match line {
        "U1" => colorize_bg(line, 22),
        "U2" => colorize_bg(line, 124),
        "U3" => colorize_bg(line, 166),
        "U4" => colorize_bg(line, 30),
        "U5" => colorize_bg(line, 94),
        "U6" => colorize_bg(line, 20),
        "U7" => {
            let mut i = line.chars();
            let lhs = i.next().unwrap();
            let rhs = i.next().unwrap();
            let lhs = Fixed(255).on(Fixed(22)).paint(format!(" {}", lhs));
            let rhs = Fixed(255).on(Fixed(124)).paint(format!("{} ", rhs));
            let total = [lhs.to_string(), rhs.to_string()].join("");
            Style::new().paint(total).to_string()
        }
        "U8" => {
            let mut i = line.chars();
            let lhs = i.next().unwrap();
            let rhs = i.next().unwrap();
            let lhs = Fixed(255).on(Fixed(124)).paint(format!(" {}", lhs));
            let rhs = Fixed(255).on(Fixed(166)).paint(format!("{} ", rhs));
            let total = [lhs.to_string(), rhs.to_string()].join("");
            Style::new().paint(total).to_string()
        }
        _ => line.to_string(),
    }
}

fn colorize_sbahn(line: &str) -> String {
    match line {
        "S1" => colorize_bg(line, 73),
        "S2" => colorize_bg(line, 34),
        "S3" => colorize_bg(line, 53),
        "S4" => colorize_bg(line, 196),
        "S6" => colorize_bg(line, 29),
        "S7" => colorize_bg(line, 204),
        "S8" => Fixed(226)
            .on(Fixed(233))
            .paint(format!(" {} ", line))
            .to_string(),
        "S20" => colorize_bg(line, 203),
        _ => line.to_string(),
    }
}

fn colorize_bg(line: &str, background_color: u8) -> String {
    Fixed(255)
        .on(Fixed(background_color))
        .paint(format!(" {} ", line))
        .to_string()
}
