use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Cell, Row, Table},
};

pub fn render_site_list(app: &mut App, frame: &mut Frame, area: Rect, block: Block) {
    let rows: Vec<Row> = app
        .sites
        .iter()
        .map(|site| {
            let device_count = site
                .devices_status
                .as_ref()
                .map(|s| s.number_of_devices)
                .unwrap_or(0);

            let mut site_color = Style::default();
            let mut lookup_key = site.name.to_lowercase();

            if let Some(vars) = &site.variables {
                for var in vars {
                    match var.name.as_str() {
                        "tuiColor" => {
                            let c = match var.value.to_lowercase().as_str() {
                                "red" => Color::Red,
                                "blue" => Color::Blue,
                                "green" => Color::Green,
                                "yellow" => Color::Yellow,
                                "magenta" => Color::Magenta,
                                "cyan" => Color::Cyan,
                                "white" => Color::White,
                                "gray" => Color::Gray,
                                _ => Color::Reset,
                            };
                            if c != Color::Reset {
                                site_color = Style::default().fg(c);
                            }
                        }
                        "tuiMdrId" => {
                            // Use the provided ID for lookup
                            lookup_key = var.value.clone();
                        }
                        _ => {}
                    }
                }
            }

            // Fetch stats using the determined key
            let stats = app
                .incident_stats
                .get(&lookup_key)
                .cloned()
                .unwrap_or_default();

            let active_style = if stats.active > 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(Span::styled(site.name.clone(), site_color)),
                Cell::from(device_count.to_string()),
                Cell::from(Span::styled(stats.active.to_string(), active_style)),
                Cell::from(stats.resolved.to_string()),
                Cell::from(site.uid.clone()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(10),
            Constraint::Percentage(10), // Active
            Constraint::Percentage(10), // Resolved
            Constraint::Percentage(40),
        ],
    )
    .header(
        Row::new(vec!["Site Name", "Devices", "Active", "Resolved", "UID"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut app.table_state);
}
