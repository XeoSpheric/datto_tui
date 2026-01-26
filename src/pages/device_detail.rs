use crate::app::{App, DeviceDetailTab};
use crate::common::utils::format_timestamp;
use crate::pages::popups::render_device_variables_popup;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs, Wrap},
};

pub fn render_device_detail(app: &mut App, frame: &mut Frame, area: Rect) {
    let selected_device_opt = app.selected_device.clone();

    if let Some(device) = selected_device_opt {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // --- Left Pane: Device Info ---
        render_device_info(&device, frame, chunks[0]);

        // --- Right Pane: Security & Activities ---
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // Security Info (Top)
                Constraint::Length(3),      // Tabs (Middle)
                Constraint::Min(0),         // Content (Bottom)
            ])
            .split(chunks[1]);

        render_device_security(app, &device, frame, right_chunks[0]);

        // Tabs
        let mut tab_titles = vec!["Open Alerts", "Activities"];
        let is_software_supported = device
            .device_class
            .as_ref()
            .map(|s| s.trim().to_lowercase())
            .as_deref()
            == Some("device");
        if is_software_supported {
            tab_titles.push("Software");
        }

        let tab_index = match app.device_detail_tab {
            DeviceDetailTab::OpenAlerts => 0,
            DeviceDetailTab::Activities => 1,
            DeviceDetailTab::Software => 2,
        };

        // Ensure tab_index is within bounds (e.g. if we switch from a device with Software to one without)
        let safe_tab_index = if tab_index >= tab_titles.len() {
            0
        } else {
            tab_index
        };

        let tabs = Tabs::new(tab_titles)
            .select(safe_tab_index)
            .block(Block::default().borders(Borders::ALL).title("View"))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            );
        frame.render_widget(tabs, right_chunks[1]);

        // Content
        match app.device_detail_tab {
            DeviceDetailTab::OpenAlerts => render_open_alerts(app, frame, right_chunks[2]),
            DeviceDetailTab::Activities => render_device_activities(app, frame, right_chunks[2]),
            DeviceDetailTab::Software => render_software(app, frame, right_chunks[2]),
        }

        // --- Variables Popup ---
        if app.show_device_variables {
            render_device_variables_popup(&device, frame, &mut app.udf_table_state);
        }
    } else {
        frame.render_widget(
            Paragraph::new("No device selected").block(Block::default().borders(Borders::ALL)),
            area,
        );
    }
}

fn render_open_alerts(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Open Alerts");

    if app.open_alerts_loading {
        frame.render_widget(Paragraph::new("Loading alerts...").block(block), area);
        return;
    }

    if let Some(err) = &app.open_alerts_error {
        frame.render_widget(
            Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(Color::Red))
                .block(block),
            area,
        );
        return;
    }

    if app.open_alerts.is_empty() {
        frame.render_widget(Paragraph::new("No open alerts.").block(block), area);
        return;
    }

    let rows: Vec<Row> = app
        .open_alerts
        .iter()
        .enumerate()
        .map(|(i, alert)| {
            let style = if Some(i) == app.open_alerts_table_state.selected() {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            let priority = alert.priority.as_deref().unwrap_or("Unknown");
            let priority_style = match priority.to_lowercase().as_str() {
                "critical" => Style::default().fg(Color::Red),
                "high" => Style::default().fg(Color::Rgb(255, 165, 0)), // Orange
                "medium" => Style::default().fg(Color::Yellow),
                "low" => Style::default().fg(Color::Blue),
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

            // Format Time
            let time_str = format_timestamp(alert.timestamp.clone());

            Row::new(vec![
                Cell::from(Span::styled(priority, priority_style)),
                Cell::from(diagnostics),
                Cell::from(time_str),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),     // Priority
            Constraint::Percentage(60), // Diagnostics
            Constraint::Length(22),     // Time
        ],
    )
    .header(
        Row::new(vec!["Priority", "Diagnostics", "Time"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(block)
    .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut app.open_alerts_table_state);
}

fn render_device_info(device: &crate::api::datto::types::Device, frame: &mut Frame, area: Rect) {
    // Format Dates
    let last_seen_str = format_timestamp(device.last_seen.clone());
    let last_reboot_str = format_timestamp(device.last_reboot.clone());
    let last_audit_str = format_timestamp(device.last_audit_date.clone());
    let creation_date_str = format_timestamp(device.creation_date.clone());

    // --- Patch Status Logic ---
    let patch_status_raw = device
        .patch_management
        .as_ref()
        .and_then(|pm| pm.patch_status.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let (patch_status_text, patch_color) = match patch_status_raw.as_str() {
        "FullyPatched" => ("Fully Patched", Color::Green),
        "ApprovedPending" => ("Approved Pending", Color::Cyan),
        "InstallError" => ("Install Error", Color::Yellow),
        "RebootRequired" => ("Reboot Required", Color::Rgb(255, 165, 0)), // Orange
        "NoData" => ("No Data", Color::Red),
        "NoPolicy" => ("No Policy", Color::Gray),
        _ => (patch_status_raw.as_str(), Color::White),
    };

    let (patches_installed, patches_pending, patches_not_approved) =
        if let Some(pm) = &device.patch_management {
            (
                pm.patches_installed.unwrap_or(0),
                pm.patches_approved_pending.unwrap_or(0),
                pm.patches_not_approved.unwrap_or(0),
            )
        } else {
            (0, 0, 0)
        };

    // --- Warranty Logic ---
    let warranty_date_str = device.warranty_date.as_deref().unwrap_or("N/A");
    let warranty_color = if warranty_date_str == "N/A" {
        Color::Red
    } else {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(warranty_date_str, "%Y-%m-%d") {
            let today = chrono::Local::now().date_naive();
            let duration = date.signed_duration_since(today);
            if duration.num_days() < 0 {
                Color::Red // Expired
            } else if duration.num_days() <= 30 {
                Color::Yellow // Coming up
            } else {
                Color::Green // OK
            }
        } else {
            Color::White // Parse error
        }
    };

    let text = vec![
        Line::from(vec![
            Span::styled(
                "Patch Status: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled("■ ", Style::default().fg(patch_color)),
            Span::raw(format!("{}", patch_status_text)),
        ]),
        Line::from(vec![Span::raw(format!(
            " | Patches Installed: {}",
            patches_installed
        ))]),
        Line::from(vec![Span::raw(format!(
            " | Patches Pending: {}",
            patches_pending
        ))]),
        Line::from(vec![Span::raw(format!(
            " | Patches Not Approved: {}",
            patches_not_approved
        ))]),
        Line::from(vec![
            Span::styled("Site: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.site_name.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw({
                let t = device
                    .device_type
                    .as_ref()
                    .and_then(|dt| dt.type_field.as_deref())
                    .unwrap_or("Unknown");
                if t == "Main System Chassis" {
                    "Server"
                } else {
                    t
                }
            }),
        ]),
        Line::from(vec![
            Span::styled("OS: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.operating_system.as_deref().unwrap_or("Unknown")),
        ]),
        Line::from(vec![
            Span::styled("Last User: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.last_logged_in_user.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(vec![
            Span::styled("IP: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "{} | {}",
                device.int_ip_address.as_deref().unwrap_or("N/A"),
                device.ext_ip_address.as_deref().unwrap_or("N/A")
            )),
        ]),
        Line::from(vec![
            Span::styled("Domain: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.domain.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(vec![
            Span::styled("Last Seen: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&last_seen_str),
        ]),
        Line::from(vec![
            Span::styled(
                "Last Reboot: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(&last_reboot_str),
        ]),
        Line::from(vec![
            Span::styled(
                "Last Audit: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(&last_audit_str),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&creation_date_str),
        ]),
        Line::from(vec![
            Span::styled("Warranty: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("■ ", Style::default().fg(warranty_color)),
            Span::raw(warranty_date_str),
        ]),
    ];

    let status_color = if device.online {
        Color::Green
    } else {
        Color::DarkGray
    };
    let status_text = if device.online { "Online" } else { "Offline" };

    let title = Line::from(vec![
        Span::raw("Device Info: "),
        Span::styled(
            &device.hostname,
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled("■ ", Style::default().fg(status_color)),
        Span::raw(status_text),
    ]);

    let info_block = Block::default().borders(Borders::ALL).title(title);

    let p = Paragraph::new(text)
        .block(info_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_device_activities(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Activities");

    if app.activity_logs_loading {
        frame.render_widget(Paragraph::new("Loading activities...").block(block), area);
        return;
    }

    if let Some(err) = &app.activity_logs_error {
        frame.render_widget(
            Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(Color::Red))
                .block(block),
            area,
        );
        return;
    }

    if app.activity_logs.is_empty() {
        frame.render_widget(Paragraph::new("No activities found.").block(block), area);
        return;
    }

    let rows: Vec<Row> = app
        .activity_logs
        .iter()
        .enumerate()
        .map(|(i, log)| {
            let style = if Some(i) == app.activity_logs_table_state.selected() {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            // Convert date (f64 timestamp) to readable string
            let date_str = format_timestamp(log.date.map(serde_json::Value::from));

            let user_name = log
                .user
                .as_ref()
                .and_then(|u| u.user_name.clone())
                .unwrap_or_else(|| "System".to_string());

            // Parse Details JSON if possible to extract Job Name and Status
            let mut job_status = String::new();
            let mut job_name = log.details.clone().unwrap_or_default();

            if let Some(details_json) = &log.details {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(details_json) {
                    if let Some(status) = parsed.get("job.status").and_then(|s| s.as_str()) {
                        job_status = status.to_string();
                    }
                    if let Some(name) = parsed.get("job.name").and_then(|s| s.as_str()) {
                        job_name = name.to_string();
                    }
                }
            }

            let status_style = match job_status.to_lowercase().as_str() {
                "expired" => Style::default().fg(Color::Rgb(255, 165, 0)), // Orange
                "scheduled" => Style::default().fg(Color::Blue),
                "running" => Style::default().fg(Color::Cyan),
                "success" => Style::default().fg(Color::Green),
                "warning" => Style::default().fg(Color::Rgb(255, 165, 0)), // Orange
                "failure" => Style::default().fg(Color::Red),
                _ => Style::default(),
            };

            Row::new(vec![
                Cell::from(date_str),
                Cell::from(job_name), // Display Job Name instead of raw details
                Cell::from(Span::styled(job_status, status_style)), // Display Status
                Cell::from(log.action.as_deref().unwrap_or("")),
                Cell::from(log.category.as_deref().unwrap_or("")),
                Cell::from(user_name),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(22),     // Time
            Constraint::Percentage(40), // Job Name
            Constraint::Length(12),     // Status
            Constraint::Length(15),     // Action
            Constraint::Length(10),     // Category
            Constraint::Length(15),     // User
        ],
    )
    .header(
        Row::new(vec![
            "Time", "Activity", "Status", "Action", "Category", "User",
        ])
        .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(block)
    .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut app.activity_logs_table_state);
}

fn render_software(app: &mut App, frame: &mut Frame, area: Rect) {
    let title = if !app.software_search_query.is_empty() || app.is_software_searching {
        format!("Software (Search: {})", app.software_search_query)
    } else {
        "Software".to_string()
    };
    let block = Block::default().borders(Borders::ALL).title(title);

    if app.device_software_loading {
        frame.render_widget(Paragraph::new("Loading software...").block(block), area);
        return;
    }

    if let Some(err) = &app.device_software_error {
        frame.render_widget(
            Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(Color::Red))
                .block(block),
            area,
        );
        return;
    }

    if app.device_software.is_empty() {
        frame.render_widget(Paragraph::new("No software found.").block(block), area);
        return;
    }

    if app.filtered_software.is_empty() && !app.software_search_query.is_empty() {
        frame.render_widget(
            Paragraph::new(format!(
                "No software matches '{}'",
                app.software_search_query
            ))
            .block(block),
            area,
        );
        return;
    }

    let rows: Vec<Row> = app
        .filtered_software
        .iter()
        .enumerate()
        .map(|(i, sw)| {
            let style = if Some(i) == app.device_software_table_state.selected() {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(sw.name.clone()),
                Cell::from(sw.version.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(70), // Name
            Constraint::Percentage(30), // Version
        ],
    )
    .header(Row::new(vec!["Name", "Version"]).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(block)
    .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut app.device_software_table_state);
}

fn render_device_security(
    app: &mut App,
    device: &crate::api::datto::types::Device,
    frame: &mut Frame,
    area: Rect,
) {
    let block = Block::default().borders(Borders::ALL).title("Security");

    let mut lines = Vec::new();

    // Determine Security Product
    let av_product_raw = device
        .antivirus
        .as_ref()
        .and_then(|av| av.antivirus_product.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("Unknown");

    let av_product_lower = av_product_raw.to_lowercase();

    // Get AV Status from Device struct (available even if detailed API call fails)
    let av_status_raw = device
        .antivirus
        .as_ref()
        .and_then(|av| av.antivirus_status.as_deref())
        .unwrap_or("Unknown");

    // Format AV Status: Split CamelCase and Color Code
    // "RunningAndUpToDate" -> "Running And Up To Date"
    let mut av_status_formatted = String::new();
    for (i, c) in av_status_raw.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            av_status_formatted.push(' ');
        }
        av_status_formatted.push(c);
    }
    // Handle special cases if needed or if regex logic was imperfect
    if av_status_formatted.is_empty() {
        av_status_formatted = "Unknown".to_string();
    }

    let av_status_color = match av_status_raw {
        "RunningAndUpToDate" => Color::Green,
        "RunningAndNotUpToDate" => Color::Yellow,
        "NotDetected" => Color::Rgb(255, 165, 0), // Orange
        "NotRunning" => Color::Red,
        _ => Color::White,
    };

    // Always show basic Product and Status
    lines.push(Line::from(vec![
        Span::styled("Product: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(av_product_raw),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(av_status_formatted, Style::default().fg(av_status_color)),
    ]));

    if av_product_lower.contains("sophos") {
        if let Some(loading) = app.sophos_loading.get(&device.hostname) {
            if *loading {
                lines.push(Line::from(Span::styled(
                    "Loading Sophos data...",
                    Style::default().fg(Color::Yellow),
                )));
            }
        }

        if let Some(endpoint) = app.sophos_endpoints.get(&device.hostname) {
            let health = endpoint
                .health
                .as_ref()
                .and_then(|h| h.overall.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("Unknown");

            let health_color = match health.to_lowercase().as_str() {
                "good" => Color::Green,
                "bad" => Color::Red,
                "suspicious" => Color::Yellow,
                _ => Color::White,
            };

            lines.push(Line::from(vec![
                Span::raw("Health: "),
                Span::styled(health, Style::default().fg(health_color)),
            ]));

            let isolated = endpoint
                .isolation
                .as_ref()
                .and_then(|i| i.is_isolated)
                .unwrap_or(false);

            lines.push(Line::from(vec![
                Span::raw("Isolation: "),
                Span::styled(
                    if isolated { "Isolated" } else { "Not Isolated" },
                    if isolated {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green)
                    },
                ),
            ]));

            if let Some(status) = app.scan_status.get(&device.hostname) {
                lines.push(Line::from(vec![
                    Span::raw("Scan Status: "),
                    Span::styled(format!("{:?}", status), Style::default().fg(Color::Cyan)),
                ]));
            }
        } else if !app
            .sophos_loading
            .get(&device.hostname)
            .cloned()
            .unwrap_or(false)
        {
            lines.push(Line::from("Detailed Sophos data not available."));
        }
    } else if av_product_lower.contains("datto av") || av_product_lower.contains("datto edr") {
        if let Some(loading) = app.datto_av_loading.get(&device.hostname) {
            if *loading {
                lines.push(Line::from(Span::styled(
                    "Loading Datto AV data...",
                    Style::default().fg(Color::Yellow),
                )));
            }
        }

        if let Some(agent) = app.datto_av_agents.get(&device.hostname) {
            lines.push(Line::from(vec![
                Span::raw("Agent Status: "),
                Span::raw(agent.status.as_deref().unwrap_or("Unknown")),
            ]));
            lines.push(Line::from(vec![
                Span::raw("Version: "),
                Span::raw(agent.version.as_deref().unwrap_or("Unknown")),
            ]));

            if let Some(status) = app.scan_status.get(&device.hostname) {
                lines.push(Line::from(vec![
                    Span::raw("Scan Status: "),
                    Span::styled(format!("{:?}", status), Style::default().fg(Color::Cyan)),
                ]));
            }
        } else if !app
            .datto_av_loading
            .get(&device.hostname)
            .cloned()
            .unwrap_or(false)
        {
            lines.push(Line::from("Detailed Datto AV data not available."));
        }
    }

    // Rocket Cyber Info
    if let Some(loading) = app.rocket_loading.get(&device.hostname) {
        if *loading {
            lines.push(Line::from(Span::styled(
                "Loading Rocket Cyber data...",
                Style::default().fg(Color::Yellow),
            )));
        }
    }

    if let Some(agent) = app.rocket_agents.get(&device.hostname) {
        lines.push(Line::from("")); // Spacer
        lines.push(Line::from(Span::styled(
            "Rocket Cyber",
            Style::default().add_modifier(Modifier::BOLD),
        )));

        let conn_color = if agent.connectivity.to_lowercase() == "online" {
            Color::Green
        } else {
            Color::Red
        };

        lines.push(Line::from(vec![
            Span::raw("Connectivity: "),
            Span::styled(&agent.connectivity, Style::default().fg(conn_color)),
        ]));

        lines.push(Line::from(vec![
            Span::raw("Agent Version: "),
            Span::raw(&agent.agent_version),
        ]));
    }

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}
