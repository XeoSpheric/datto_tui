use crate::app::{App, SiteDetailTab};
use crate::common::utils::draw_pie_chart;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs, Wrap},
};

pub fn render_site_detail(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // --- Left Pane: Site Details ---
    if let Some(idx) = app.table_state.selected() {
        if let Some(site) = app.sites.get(idx) {
            let chart_height = (chunks[0].width / 3) / 2;
            let chart_height = chart_height.max(10).min(25); // Sanity bounds

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(10),
                    Constraint::Length(chart_height),
                    Constraint::Min(0),
                ])
                .split(chunks[0]);

            let text = vec![
                Line::from(vec![
                    Span::styled(
                        "Description: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(site.description.as_deref().unwrap_or("N/A")),
                ]),
                Line::from(vec![
                    Span::styled("Devices: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(
                        site.devices_status
                            .as_ref()
                            .map_or("0".to_string(), |s| s.number_of_devices.to_string()),
                    ),
                ]),
            ];

            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Site: {}", site.name));
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, left_chunks[0]);

            // Pie Charts Area
            let charts_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .split(left_chunks[1]);

            render_alerts_pie(app, frame, charts_layout[0]);
            render_devices_pie(app, frame, charts_layout[1]);
            render_patch_pie(app, frame, charts_layout[2]);

            render_av_status_bar_chart(app, frame, left_chunks[2]);
        }
    }

    // --- Right Pane: Content (Tabs) ---
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[1]);

    let tabs = Tabs::new(vec!["Devices", "Alerts", "Variables", "Settings"])
        .select(match app.detail_tab {
            SiteDetailTab::Devices => 0,
            SiteDetailTab::Alerts => 1,
            SiteDetailTab::Variables => 2,
            SiteDetailTab::Settings => 3,
        })
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        );
    frame.render_widget(tabs, right_chunks[0]);

    match app.detail_tab {
        SiteDetailTab::Devices => render_device_list(app, frame, right_chunks[1]),
        SiteDetailTab::Alerts => render_site_alerts(app, frame, right_chunks[1]),
        SiteDetailTab::Variables => render_variables(app, frame, right_chunks[1]),
        SiteDetailTab::Settings => render_settings(app, frame, right_chunks[1]),
    }
}

fn render_settings(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Settings ('Space/Enter': toggle/edit)");

    // Define the rows for the settings table
    let rows = vec![
        Row::new(vec![
            Cell::from("Name"),
            Cell::from(app.site_edit_state.name.clone()),
        ]),
        Row::new(vec![
            Cell::from("Description"),
            Cell::from(app.site_edit_state.description.clone()),
        ]),
        Row::new(vec![
            Cell::from("Notes"),
            Cell::from(app.site_edit_state.notes.clone()),
        ]),
        Row::new(vec![
            Cell::from("On Demand"),
            Cell::from(if app.site_edit_state.on_demand {
                "[x] Enabled"
            } else {
                "[ ] Disabled"
            }),
        ]),
        Row::new(vec![
            Cell::from("Splashtop Auto-Install"),
            Cell::from(if app.site_edit_state.splashtop_auto_install {
                "[x] Enabled"
            } else {
                "[ ] Disabled"
            }),
        ]),
    ];

    let table = Table::new(
        rows,
        [Constraint::Percentage(30), Constraint::Percentage(70)],
    )
    .header(Row::new(vec!["Setting", "Value"]).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut app.settings_table_state);
}

fn render_device_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let devices_block = Block::default().borders(Borders::ALL).title("Devices");

    if app.devices_loading {
        frame.render_widget(
            Paragraph::new("Loading devices...").block(devices_block),
            area,
        );
    } else if let Some(err) = &app.devices_error {
        frame.render_widget(
            Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(Color::Red))
                .block(devices_block),
            area,
        );
    } else {
        let rows: Vec<Row> = app
            .devices
            .iter()
            .enumerate()
            .map(|(i, device)| {
                let style = if Some(i) == app.devices_table_state.selected() {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };

                let status = if device.online { "Online" } else { "Offline" };
                let status_color = if device.online {
                    Color::Green
                } else {
                    Color::Gray
                };

                let patch_status = device
                    .patch_management
                    .as_ref()
                    .and_then(|pm| pm.patch_status.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                let patch_color = match patch_status.as_str() {
                    "FullyPatched" => Color::Green,
                    "ApprovedPending" => Color::Cyan, // Light Green/Cyan
                    "NoPolicy" => Color::Red,
                    "NoData" => Color::Magenta,
                    "RebootRequired" => Color::LightRed, // Orange-ish often represented by LightRed or Yellow
                    "InstallError" => Color::Yellow,
                    _ => Color::Gray,
                };

                let mut device_type = device
                    .device_type
                    .as_ref()
                    .and_then(|dt| dt.type_field.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                if device_type == "Main System Chassis" {
                    device_type = "Server".to_string();
                }

                let hostname_prefix = if app.selected_device_uids.contains(&device.uid) {
                    "[*] "
                } else {
                    ""
                };

                Row::new(vec![
                    Cell::from(format!("{}{}", hostname_prefix, device.hostname)),
                    Cell::from(device_type),
                    Cell::from(Span::styled(status, Style::default().fg(status_color))),
                    Cell::from(Span::styled(patch_status, Style::default().fg(patch_color))),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(35),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(35),
            ],
        )
        .header(
            Row::new(vec!["Hostname", "Type", "Status", "Patch Status"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(devices_block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(table, area, &mut app.devices_table_state);
    }
}

fn render_site_alerts(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Site Alerts");

    if app.site_open_alerts_loading {
        frame.render_widget(Paragraph::new("Loading alerts...").block(block), area);
        return;
    }

    if let Some(err) = &app.site_open_alerts_error {
        frame.render_widget(
            Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(Color::Red))
                .block(block),
            area,
        );
        return;
    }

    if app.site_open_alerts.is_empty() {
        frame.render_widget(Paragraph::new("No open alerts.").block(block), area);
        return;
    }

    let rows: Vec<Row> = app
        .site_open_alerts
        .iter()
        .enumerate()
        .map(|(i, alert)| {
            let style = if Some(i) == app.site_open_alerts_table_state.selected() {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            let priority = alert.priority.as_deref().unwrap_or("Unknown");
            let priority_style = match priority.to_lowercase().as_str() {
                "critical" => Style::default().fg(Color::Red),
                "high" => Style::default().fg(Color::Rgb(255, 165, 0)), // Orange
                "moderate" | "medium" => Style::default().fg(Color::Yellow),
                "low" => Style::default().fg(Color::Cyan),
                "information" => Style::default().fg(Color::White),
                _ => Style::default(),
            };

            let diagnostics = alert
                .diagnostics
                .as_deref()
                .unwrap_or("N/A")
                .replace("\r\n", " ")
                .replace('\n', " ")
                .trim()
                .to_string();

            let computer_name = alert
                .alert_source_info
                .as_ref()
                .and_then(|s| s.device_name.as_deref())
                .unwrap_or("N/A");

            Row::new(vec![
                Cell::from(Span::styled(priority, priority_style)),
                Cell::from(diagnostics),
                Cell::from(computer_name.to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),     // Priority
            Constraint::Percentage(60), // Diagnostics
            Constraint::Percentage(25), // Computer Name
        ],
    )
    .header(
        Row::new(vec!["Priority", "Diagnostics", "Computer Name"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(block)
    .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut app.site_open_alerts_table_state);
}

fn render_variables(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Variables (Space/Enter: Select)");

    if let Some(idx) = app.table_state.selected() {
        if let Some(site) = app.sites.get(idx) {
            if let Some(vars) = &site.variables {
                let mut rows: Vec<Row> = vars
                    .iter()
                    .enumerate()
                    .map(|(i, var)| {
                        let style = if Some(i) == app.variables_table_state.selected() {
                            Style::default().add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        };

                        Row::new(vec![
                            Cell::from(var.name.clone()),
                            Cell::from(var.value.clone()),
                            Cell::from(if var.masked { "*" } else { "" }),
                        ])
                        .style(style)
                    })
                    .collect();

                // Add "Create new variable" row
                rows.push(
                    Row::new(vec![
                        Cell::from(Span::styled(
                            "+ Create new",
                            Style::default().add_modifier(Modifier::BOLD | Modifier::ITALIC),
                        )),
                        Cell::from(""),
                        Cell::from(""),
                    ])
                    .style(
                        if app.variables_table_state.selected() == Some(rows.len()) {
                            Style::default().add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        },
                    ),
                );

                let table = Table::new(
                    rows,
                    [
                        Constraint::Percentage(30),
                        Constraint::Percentage(60),
                        Constraint::Percentage(10),
                    ],
                )
                .header(
                    Row::new(vec!["Name", "Value", "Masked"])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(block) // Use the block here
                .highlight_symbol(">> ");

                frame.render_stateful_widget(table, area, &mut app.variables_table_state);
                return;
            }
        }
    }
    // Fallback if no site selected or no variables
    frame.render_widget(Paragraph::new("No variables").block(block), area);
}

fn render_alerts_pie(app: &App, frame: &mut Frame, area: Rect) {
    let mut info = 0;
    let mut low = 0;
    let mut moderate = 0;
    let mut high = 0;
    let mut critical = 0;

    for alert in &app.site_open_alerts {
        match alert
            .priority
            .as_deref()
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("information") => info += 1,
            Some("low") => low += 1,
            Some("moderate") | Some("medium") => moderate += 1,
            Some("high") => high += 1,
            Some("critical") => critical += 1,
            _ => {}
        }
    }

    let total = info + low + moderate + high + critical;
    let data = vec![
        (info as f64, Color::White, "Info"),
        (low as f64, Color::Cyan, "Low"),
        (moderate as f64, Color::Yellow, "Mod"),
        (high as f64, Color::Rgb(255, 165, 0), "High"),
        (critical as f64, Color::Red, "Crit"),
    ];

    draw_pie_chart(frame, area, "Open Alerts", total, &data);
}

fn render_devices_pie(app: &App, frame: &mut Frame, area: Rect) {
    let mut online = 0;
    let mut offline = 0;

    if let Some(idx) = app.table_state.selected() {
        if let Some(site) = app.sites.get(idx) {
            if let Some(status) = &site.devices_status {
                online = status.number_of_online_devices;
                offline = status.number_of_offline_devices;
            }
        }
    }

    let total = online + offline;
    let data = vec![
        (online as f64, Color::Green, "Online"),
        (offline as f64, Color::Red, "Offline"),
    ];

    draw_pie_chart(frame, area, "Device Status", total, &data);
}

fn render_patch_pie(app: &App, frame: &mut Frame, area: Rect) {
    let mut fully_patched = 0;
    let mut approved_pending = 0;
    let mut install_error = 0;
    let mut reboot_required = 0;
    let mut no_data = 0;
    let mut no_policy = 0;
    let mut other = 0;

    for device in &app.devices {
        if let Some(pm) = &device.patch_management {
            match pm.patch_status.as_deref() {
                Some("FullyPatched") => fully_patched += 1,
                Some("ApprovedPending") => approved_pending += 1,
                Some("InstallError") => install_error += 1,
                Some("RebootRequired") => reboot_required += 1,
                Some("NoData") => no_data += 1,
                Some("NoPolicy") => no_policy += 1,
                _ => other += 1,
            }
        } else {
            other += 1;
        }
    }

    let total = fully_patched
        + approved_pending
        + install_error
        + reboot_required
        + no_data
        + no_policy
        + other;

    let data = vec![
        (fully_patched as f64, Color::Green, "Patched"),
        (approved_pending as f64, Color::Cyan, "Pending"),
        (install_error as f64, Color::Yellow, "Error"),
        (reboot_required as f64, Color::Rgb(255, 165, 0), "Reboot"),
        (no_data as f64, Color::Red, "No Data"),
        (no_policy as f64, Color::Gray, "No Pol"),
        (other as f64, Color::White, "Other"),
    ];

    draw_pie_chart(frame, area, "Patch Status", total, &data);
}

fn render_av_status_bar_chart(app: &App, frame: &mut Frame, area: Rect) {
    let mut stats: std::collections::BTreeMap<String, i32> = std::collections::BTreeMap::new();
    for device in &app.devices {
        let status = device
            .antivirus
            .as_ref()
            .and_then(|av| av.antivirus_status.as_deref())
            .unwrap_or("Unknown");
        *stats.entry(status.to_string()).or_insert(0) += 1;
    }

    let mut lines = Vec::new();
    let max_value = stats.values().cloned().max().unwrap_or(1);

    // Reserve more space for labels and counts to prevent cutoff
    // Label takes up to ~25 chars, count up to ~6, plus borders
    let reserved_width = 35;
    let bar_max_width = (area.width as i32 - reserved_width).max(1) as usize;

    for (status_raw, count) in stats {
        // Format status: RunningAndUpToDate -> Running And Up To Date
        let mut status_formatted = String::new();
        for (i, c) in status_raw.chars().enumerate() {
            if i > 0 && c.is_uppercase() {
                status_formatted.push(' ');
            }
            status_formatted.push(c);
        }

        let bar_width = ((count as f64 / max_value as f64) * bar_max_width as f64) as usize;
        let bar = "█".repeat(bar_width);

        let color = match status_raw.as_str() {
            "RunningAndUpToDate" => Color::Green,
            "RunningAndNotUpToDate" => Color::Yellow,
            "NotDetected" => Color::Rgb(255, 165, 0), // Orange
            "NotRunning" => Color::Red,
            _ => Color::White,
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>24}: ", status_formatted),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(bar, Style::default().fg(color)),
            Span::raw(format!(" {}", count)),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Anti-Virus Status");
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
