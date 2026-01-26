use chrono::DateTime;
use ratatui::{
    prelude::*,
    widgets::{
        canvas::{Canvas, Line as CanvasLine},
        Block, Borders,
    },
};

/// Formats a timestamp from a serde_json::Value (either milliseconds or ISO string)
/// into a human-readable date/time string in the Central US timezone.
///
/// # Arguments
/// * `ts_option` - An Option containing a serde_json::Value representing the timestamp.
///
/// # Returns
/// A formatted string "MM/DD/YYYY HH:MMam/pm" or "N/A" if invalid.
pub fn format_timestamp(ts_option: Option<serde_json::Value>) -> String {
    if let Some(val) = ts_option {
        if let Some(ts_f64) = val.as_f64() {
            // Check if milliseconds (likely) or seconds
            // 2026 timestamp: 1768448871000 is definitely millis (13 digits)
            // Anything > 10,000,000,000 is likely millis
            let (seconds, nanoseconds) = if ts_f64 > 10_000_000_000.0 {
                let s = (ts_f64 / 1000.0) as i64;
                let n = ((ts_f64 % 1000.0) * 1_000_000.0) as u32;
                (s, n)
            } else {
                let s = ts_f64 as i64;
                let n = ((ts_f64 - s as f64) * 1_000_000_000.0) as u32;
                (s, n)
            };

            if let Some(dt) = DateTime::from_timestamp(seconds, nanoseconds) {
                let local_dt = dt.with_timezone(&chrono::Local);
                return local_dt.format("%m/%d/%Y %I:%M%P").to_string();
            }
        } else if let Some(s) = val.as_str() {
            // Try to parse ISO string
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                let local_dt = dt.with_timezone(&chrono::Local);
                return local_dt.format("%m/%d/%Y %I:%M%P").to_string();
            }
            return s.to_string();
        }
    }
    "N/A".to_string()
}

/// Calculates a centered rectangle of a given percentage size within another Rect.
/// Useful for displaying popups/modals in the center of the screen.
///
/// # Arguments
/// * `percent_x` - Horizontal percentage of the screen the rect should occupy.
/// * `percent_y` - Vertical percentage of the screen the rect should occupy.
/// * `r` - The parent Rect (usually the full frame area).
///
/// # Returns
/// A new Rect centered within the parent Rect.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let hor_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    hor_layout[1]
}

/// Draws a pie chart on a given frame area using the Ratatui Canvas widget.
///
/// # Arguments
/// * `frame` - The TUI Frame to render into.
/// * `area` - The Rect area dedicated to the chart.
/// * `title` - The title displayed on the chart's block border.
/// * `total` - The sum of all values in the data.
/// * `data` - A slice of tuples containing (value, color, label).
pub fn draw_pie_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    total: i32,
    data: &[(f64, Color, &str)],
) {
    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .x_bounds([-100.0, 100.0])
        .y_bounds([-100.0, 100.0])
        .paint(move |ctx| {
            if total == 0 {
                ctx.print(
                    0.0,
                    0.0,
                    Span::styled("No Data", Style::default().fg(Color::DarkGray)),
                );
                return;
            }

            let mut current_angle = 0.0;
            // Adjust radius to maintain circular shape based on area aspect ratio
            // Terminal characters are roughly 2x taller than wide.
            let width = area.width as f64;
            let height = area.height as f64;
            let radius_x = 80.0;
            let radius_y = radius_x * (width / (height * 2.0));

            // Ensure radius_y doesn't exceed canvas bounds
            let radius_y = radius_y.min(90.0);
            let radius_x = if radius_y == 90.0 {
                90.0 * (height * 2.0 / width)
            } else {
                radius_x
            };

            for (value, color, _label) in data {
                if *value <= 0.0 {
                    continue;
                }

                let sweep = (*value / total as f64) * 2.0 * std::f64::consts::PI;
                let end_angle = current_angle + sweep;

                let steps = (sweep * 100.0) as i32; // density
                for i in 0..=steps {
                    let angle = current_angle + (sweep * i as f64 / steps as f64);
                    let x = radius_x * angle.cos();
                    let y = radius_y * angle.sin();
                    ctx.draw(&CanvasLine {
                        x1: 0.0,
                        y1: 0.0,
                        x2: x,
                        y2: y,
                        color: *color,
                    });
                }
                current_angle = end_angle;
            }

            let total_str = total.to_string();
            ctx.print(
                0.0,
                0.0,
                Span::styled(
                    total_str,
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::White)
                        .bg(Color::Black),
                ),
            );
        });

    frame.render_widget(canvas, area);
}

/// Opens a URL in the default web browser in a cross-platform way.
///
/// # Arguments
/// * `url` - The URL string to open.
pub fn open_browser(url: &str) {
    let result = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).spawn()
    } else {
        // Assume Linux/Unix
        std::process::Command::new("xdg-open").arg(url).spawn()
    };

    if let Err(e) = result {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(f, "Failed to open browser: {}", e).unwrap();
            });
    }
}
