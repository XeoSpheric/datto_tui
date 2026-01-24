use crate::app::{
    App, CurrentView, DeviceDetailTab, InputField, InputMode, JobViewRow, SiteDetailTab,
};
use crate::app_helpers::generate_job_rows;
use chrono::DateTime;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Tabs, Wrap},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    // Title / Status
    let status_text = match app.current_view {
        CurrentView::List => {
            format!(
                "Kyber TUI | Sites: {} | 'q': quit, 'r': reload, '/': search devices, 'j/k': move, 'Enter': details",
                app.total_count
            )
        }
        CurrentView::Detail => "Detail View | 'Esc'/'q': back, '/': search devices".to_string(),
        CurrentView::DeviceDetail => {
            "Device Detail | 'Esc'/'q': back, '/': search devices".to_string()
        }
        CurrentView::ActivityDetail => "Activity Detail | 'Esc'/'q': back".to_string(),
    };

    frame.render_widget(
        Paragraph::new(status_text).block(Block::default().borders(Borders::ALL).title("Status")),
        layout[0],
    );

    // Main Content
    let main_block = Block::default().borders(Borders::ALL).title("Sites");

    if let Some(err) = &app.error {
        frame.render_widget(
            Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(Color::Red))
                .block(main_block),
            layout[1],
        );
    } else if app.is_loading {
        frame.render_widget(
            Paragraph::new("Loading sites...")
                .style(Style::default().fg(Color::Yellow))
                .block(main_block),
            layout[1],
        );
    } else {
        match app.current_view {
            CurrentView::List => render_list(app, frame, layout[1], main_block),
            CurrentView::Detail => render_detail(app, frame, layout[1]),
            CurrentView::DeviceDetail => render_device_detail(app, frame, layout[1]),
            CurrentView::ActivityDetail => render_activity_detail(app, frame, layout[1]),
        }
    }

    // Render Input Modal if Editing
    if app.input_state.mode == InputMode::Editing {
        render_input_modal(app, frame);
    }

    // Render Popup
    render_popup(app, frame);

    // Render Device Search Popup
    if app.show_device_search {
        render_device_search_popup(app, frame);
    }
}

fn render_list(app: &mut App, frame: &mut Frame, area: Rect, block: Block) {
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

fn render_detail(app: &mut App, frame: &mut Frame, area: Rect) {
    // Split the detail area into two vertical chunks: Top for Site Info (50%), Bottom for Device List (50%)
    // Or Horizontal: Left for Site Info, Right for Devices. User interaction "details of site on left and list of devices on right"

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // --- Left Pane: Site Details ---
    if let Some(idx) = app.table_state.selected() {
        if let Some(site) = app.sites.get(idx) {
            let text = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&site.name),
                ]),
                Line::from(vec![
                    Span::styled("UID: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&site.uid),
                ]),
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
                Line::from(vec![
                    Span::styled("Online: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(
                        site.devices_status
                            .as_ref()
                            .map_or("0".to_string(), |s| s.number_of_online_devices.to_string()),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Offline: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(
                        site.devices_status
                            .as_ref()
                            .map_or("0".to_string(), |s| s.number_of_offline_devices.to_string()),
                    ),
                ]),
            ];

            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Site: {}", site.name));
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, chunks[0]);
        }
    }

    // --- Right Pane: Content (Tabs) ---
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[1]);

    let tabs = Tabs::new(vec!["Devices", "Variables", "Settings"])
        .select(match app.detail_tab {
            SiteDetailTab::Devices => 0,
            SiteDetailTab::Variables => 1,
            SiteDetailTab::Settings => 2,
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

                Row::new(vec![
                    Cell::from(device.hostname.clone()),
                    Cell::from(Span::styled(status, Style::default().fg(status_color))),
                    Cell::from(Span::styled(patch_status, Style::default().fg(patch_color))),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ],
        )
        .header(
            Row::new(vec!["Hostname", "Status", "Patch Status"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(devices_block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(table, area, &mut app.devices_table_state);
    }
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

fn render_input_modal(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area); // Clear background

    let (title, is_single_field_edit) = if let Some(field) = &app.input_state.editing_setting {
        (format!("Edit Setting: {:?}", field), true) // Primitive debug format for now, can perform better mapping later
    } else if let Some(idx) = app.editing_udf_index {
        (format!("Edit UDF {}", idx + 1), true)
    } else if app.input_state.is_creating {
        ("Create Variable".to_string(), false)
    } else {
        ("Edit Variable".to_string(), false)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(block, area);

    // If it's a settings edit or UDF edit, we only need one input field. If variable, we might need two (Name/Value)
    let constraints = if is_single_field_edit {
        vec![
            Constraint::Length(3), // Value (or Name, generic input)
            Constraint::Min(0),    // Instructions
        ]
    } else {
        // Variable edit/create needs two fields
        vec![
            Constraint::Length(3), // Name
            Constraint::Length(3), // Value
            Constraint::Min(0),    // Instructions
        ]
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(area);

    if is_single_field_edit {
        // Single input field for settings or UDF
        // For UDFs, we use the value_buffer. For settings, we act funny and use name_buffer currently (based on previous code)
        // Let's check which buffer to display.
        let (buffer, label) = if app.editing_udf_index.is_some() {
            (app.input_state.value_buffer.clone(), "Value")
        } else {
            (app.input_state.name_buffer.clone(), "Value")
        };

        let input_style = Style::default().fg(Color::Yellow);
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(label)
            .style(input_style);
        let input_text = Paragraph::new(buffer).block(input_block);
        frame.render_widget(input_text, layout[0]);

        let instructions =
            Paragraph::new("Enter: submit | Esc: cancel").alignment(Alignment::Center);
        frame.render_widget(instructions, layout[1]);
    } else {
        // Variable Edit (Original Logic)
        // Name Input
        let name_style = if app.input_state.active_field == InputField::Name {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let name_block = Block::default()
            .borders(Borders::ALL)
            .title("Name")
            .style(name_style);
        let name_text = Paragraph::new(app.input_state.name_buffer.clone()).block(name_block);
        frame.render_widget(name_text, layout[0]);

        // Value Input
        let value_style = if app.input_state.active_field == InputField::Value {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let value_block = Block::default()
            .borders(Borders::ALL)
            .title("Value")
            .style(value_style);
        let value_text = Paragraph::new(app.input_state.value_buffer.clone()).block(value_block);
        frame.render_widget(value_text, layout[1]);

        // Instructions
        let instructions = Paragraph::new("Tab: switch field | Enter: submit | Esc: cancel")
            .alignment(Alignment::Center);
        frame.render_widget(instructions, layout[2]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn render_device_detail(app: &mut App, frame: &mut Frame, area: Rect) {
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
        let tabs = Tabs::new(vec!["Open Alerts", "Activities"])
            .select(match app.device_detail_tab {
                DeviceDetailTab::OpenAlerts => 0,
                DeviceDetailTab::Activities => 1,
            })
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
            Span::raw(format!(
                "{} | Patches Installed: {} | Patches Pending: {} | Patches Not Approved: {}",
                patch_status_text, patches_installed, patches_pending, patches_not_approved
            )),
        ]),
        Line::from(vec![
            Span::styled("Site: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.site_name.as_deref().unwrap_or("N/A")),
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

fn format_timestamp(ts_option: Option<serde_json::Value>) -> String {
    if let Some(val) = ts_option {
        if let Some(ts_i64) = val.as_i64() {
            // Check if milliseconds (likely) or seconds
            // 2026 timestamp: 1768448871000 is definitely millis (13 digits)
            let seconds = ts_i64 / 1000;
            let nanoseconds = ((ts_i64 % 1000) * 1_000_000) as u32;

            if let Some(dt) = DateTime::from_timestamp(seconds, nanoseconds) {
                return dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            }
        } else if let Some(s) = val.as_str() {
            // Try to parse ISO string
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                return dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            }
            return s.to_string();
        }
    }
    "N/A".to_string()
}

fn render_device_variables_popup(
    device: &crate::api::datto::types::Device,
    frame: &mut Frame,
    state: &mut TableState,
) {
    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Variables (UDF) - Press 'Enter' to Edit | 'Esc'/'v' to close")
        .style(Style::default().bg(Color::DarkGray));

    let mut rows = Vec::new();

    if let Some(udf) = &device.udf {
        let udfs = vec![
            ("UDF 1", &udf.udf1),
            ("UDF 2", &udf.udf2),
            ("UDF 3", &udf.udf3),
            ("UDF 4", &udf.udf4),
            ("UDF 5", &udf.udf5),
            ("UDF 6", &udf.udf6),
            ("UDF 7", &udf.udf7),
            ("UDF 8", &udf.udf8),
            ("UDF 9", &udf.udf9),
            ("UDF 10", &udf.udf10),
            ("UDF 11", &udf.udf11),
            ("UDF 12", &udf.udf12),
            ("UDF 13", &udf.udf13),
            ("UDF 14", &udf.udf14),
            ("UDF 15", &udf.udf15),
            ("UDF 16", &udf.udf16),
            ("UDF 17", &udf.udf17),
            ("UDF 18", &udf.udf18),
            ("UDF 19", &udf.udf19),
            ("UDF 20", &udf.udf20),
            ("UDF 21", &udf.udf21),
            ("UDF 22", &udf.udf22),
            ("UDF 23", &udf.udf23),
            ("UDF 24", &udf.udf24),
            ("UDF 25", &udf.udf25),
            ("UDF 26", &udf.udf26),
            ("UDF 27", &udf.udf27),
            ("UDF 28", &udf.udf28),
            ("UDF 29", &udf.udf29),
            ("UDF 30", &udf.udf30),
        ];

        for (label, val_opt) in udfs {
            let val = val_opt.as_deref().unwrap_or("");
            rows.push(Row::new(vec![Cell::from(label), Cell::from(val)]));
        }
    } else {
        // Even if no UDF object, show empty slots so user can edit them
        for i in 1..=30 {
            rows.push(Row::new(vec![
                Cell::from(format!("UDF {}", i)),
                Cell::from(""),
            ]));
        }
    }

    let table = Table::new(
        rows,
        [Constraint::Percentage(30), Constraint::Percentage(70)],
    )
    .header(Row::new(vec!["Field", "Value"]).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(block)
    .highlight_symbol(">> ")
    .row_highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow),
    );

    frame.render_stateful_widget(table, area, state);
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
            let date_str = if let Some(ts) = log.date {
                let seconds = ts as i64;
                let nanos = ((ts - seconds as f64) * 1_000_000_000.0) as u32;
                if let Some(dt) = DateTime::from_timestamp(seconds, nanos) {
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                } else {
                    ts.to_string()
                }
            } else {
                "N/A".to_string()
            };

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

fn render_device_security(
    app: &mut App,
    device: &crate::api::datto::types::Device,
    frame: &mut Frame,
    area: Rect,
) {
    let block = Block::default().borders(Borders::ALL).title("Security");

    let mut lines = Vec::new();

    // Determine Security Product
    let av_product = device
        .antivirus
        .as_ref()
        .and_then(|av| av.antivirus_product.as_ref())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

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

    if av_product.contains("sophos") {
        if let Some(loading) = app.sophos_loading.get(&device.hostname) {
            if *loading {
                lines.push(Line::from(Span::styled(
                    "Loading Sophos data...",
                    Style::default().fg(Color::Yellow),
                )));
            }
        }

        // Always show Product and AV Status
        lines.push(Line::from(vec![Span::styled(
            "Product: Sophos Endpoint",
            Style::default().add_modifier(Modifier::BOLD),
        )]));

        lines.push(Line::from(vec![
            Span::raw("Status: "),
            Span::styled(av_status_formatted, Style::default().fg(av_status_color)),
        ]));

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
        } else if lines.len() <= 2 {
            // Only Product and Status shown, no detailed data yet
            lines.push(Line::from("Detailed data not available."));
        }
    } else if av_product.contains("datto") {
        if let Some(loading) = app.datto_av_loading.get(&device.hostname) {
            if *loading {
                lines.push(Line::from(Span::styled(
                    "Loading Datto AV data...",
                    Style::default().fg(Color::Yellow),
                )));
            }
        }

        // Always show Product and AV Status
        lines.push(Line::from(vec![Span::styled(
            "Product: Datto AV",
            Style::default().add_modifier(Modifier::BOLD),
        )]));

        lines.push(Line::from(vec![
            Span::raw("Status: "),
            Span::styled(av_status_formatted, Style::default().fg(av_status_color)),
        ]));

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
        } else if lines.len() <= 2 {
            lines.push(Line::from("Detailed data not available."));
        }
    } else {
        lines.push(Line::from("No supported security product detected."));
    }

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_activity_detail(app: &mut App, frame: &mut Frame, area: Rect) {
    if let Some(log) = &app.selected_activity_log {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Activity Log Details");

        // Format date
        let date_str = if let Some(ts) = log.date {
            let seconds = ts as i64;
            let nanos = ((ts - seconds as f64) * 1_000_000_000.0) as u32;
            if let Some(dt) = DateTime::from_timestamp(seconds, nanos) {
                dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
            } else {
                ts.to_string()
            }
        } else {
            "N/A".to_string()
        };

        let user_name = log
            .user
            .as_ref()
            .and_then(|u| u.user_name.clone())
            .unwrap_or_else(|| "System".to_string());

        let site_name = log
            .site
            .as_ref()
            .and_then(|s| s.name.clone())
            .unwrap_or_else(|| "Unknown Site".to_string());

        // Parse Details JSON
        let mut job_status = String::new();
        let mut job_name = String::new();
        let mut extra_details = Vec::new();

        if let Some(details_json) = &log.details {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(details_json) {
                if let Some(obj) = parsed.as_object() {
                    for (k, v) in obj {
                        if k == "job.status" {
                            job_status = v.as_str().unwrap_or("").to_string();
                        } else if k == "job.name" {
                            job_name = v.as_str().unwrap_or("").to_string();
                        } else {
                            // Format other keys nicely
                            let val_str = if let Some(s) = v.as_str() {
                                s.to_string()
                            } else {
                                v.to_string()
                            };
                            extra_details.push((k.clone(), val_str));
                        }
                    }
                }
            }
        }

        // Sort extra details for consistent display
        extra_details.sort_by(|a, b| a.0.cmp(&b.0));

        let status_style = match job_status.to_lowercase().as_str() {
            "expired" => Style::default().fg(Color::Rgb(255, 165, 0)), // Orange
            "scheduled" => Style::default().fg(Color::Blue),
            "running" => Style::default().fg(Color::Cyan),
            "success" => Style::default().fg(Color::Green),
            "warning" => Style::default().fg(Color::Rgb(255, 165, 0)), // Orange
            "failure" => Style::default().fg(Color::Red),
            _ => Style::default(),
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Time: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(date_str),
            ]),
            Line::from(vec![
                Span::styled("Job Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if !job_name.is_empty() {
                    job_name.clone()
                } else {
                    "N/A".to_string()
                }),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    if !job_status.is_empty() {
                        job_status
                    } else {
                        "N/A".to_string()
                    },
                    status_style,
                ),
            ]),
            Line::from(vec![
                Span::styled("Action: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(log.action.as_deref().unwrap_or("N/A")),
            ]),
            Line::from(vec![
                Span::styled("Category: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(log.category.as_deref().unwrap_or("N/A")),
            ]),
            Line::from(vec![
                Span::styled("User: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(user_name),
            ]),
            Line::from(vec![
                Span::styled("Entity: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(log.entity.as_deref().unwrap_or("N/A")),
            ]),
            Line::from(vec![
                Span::styled("Hostname: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(log.hostname.as_deref().unwrap_or("N/A")),
            ]),
            Line::from(vec![
                Span::styled("Site: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(site_name),
            ]),
            Line::from(""),
        ];

        // Job Results Section
        if app.job_result_loading {
            lines.push(Line::from(Span::styled(
                "Loading Job Results...",
                Style::default().fg(Color::Yellow),
            )));
        } else if let Some(err) = &app.job_result_error {
            lines.push(Line::from(Span::styled(
                format!("Error fetching job results: {}", err),
                Style::default().fg(Color::Red),
            )));
        } else if let Some(job_result) = &app.selected_job_result {
            lines.push(Line::from(Span::styled(
                "Job Results:",
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));

            let status = job_result.job_deployment_status.as_deref().unwrap_or("N/A");
            let deployment_status_color = match status.to_lowercase().as_str() {
                "success" => Color::Green,
                "failure" | "error" => Color::Red,
                "warning" | "expired" => Color::Rgb(255, 165, 0), // Orange
                "scheduled" => Color::Blue,
                "running" => Color::Cyan,
                _ => Color::White,
            };

            lines.push(Line::from(vec![
                Span::styled(
                    "Deployment Status: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(status, Style::default().fg(deployment_status_color)),
            ]));
            let ran_on_str = match &job_result.ran_on {
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(serde_json::Value::Number(n)) => {
                    if let Some(ts) = n.as_i64() {
                        let seconds = ts / 1000;
                        let nanos = ((ts % 1000) * 1_000_000) as u32;
                        if let Some(dt) = DateTime::from_timestamp(seconds, i64::from(nanos) as u32)
                        {
                            dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                        } else {
                            ts.to_string()
                        }
                    } else if let Some(f) = n.as_f64() {
                        // Assuming seconds.fraction if float, or milliseconds?
                        // If it's like 1.7e12, it's millis. If 1.7e9, it's seconds.
                        // The example 1769126162000 is millis.
                        if f > 10_000_000_000.0 {
                            // Milliseconds
                            let seconds = (f / 1000.0) as i64;
                            let nanos = ((f % 1000.0) * 1_000_000.0) as u32;
                            if let Some(dt) = DateTime::from_timestamp(seconds, nanos) {
                                dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                            } else {
                                f.to_string()
                            }
                        } else {
                            // Seconds
                            let seconds = f as i64;
                            let nanos = ((f - seconds as f64) * 1_000_000_000.0) as u32;
                            if let Some(dt) = DateTime::from_timestamp(seconds, nanos) {
                                dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                            } else {
                                f.to_string()
                            }
                        }
                    } else {
                        n.to_string()
                    }
                }
                _ => "N/A".to_string(),
            };

            lines.push(Line::from(vec![
                Span::styled("Ran On: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(ran_on_str),
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Components:",
                Style::default().add_modifier(Modifier::BOLD),
            )));

            if let Some(components) = &job_result.component_results {
                let rows = generate_job_rows(job_result);
                // We need to match displayed lines to rows.
                // iterate rows and render.

                for (row_index, row) in rows.iter().enumerate() {
                    let is_selected = row_index == app.selected_job_row_index;
                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        Style::default()
                    };

                    match row {
                        JobViewRow::ComponentHeader(idx) => {
                            if let Some(comp) = components.get(*idx) {
                                let status_color = match comp
                                    .component_status
                                    .as_deref()
                                    .unwrap_or("")
                                    .to_lowercase()
                                    .as_str()
                                {
                                    "success" => Color::Green,
                                    "failure" | "error" => Color::Red,
                                    "warning" => Color::Yellow,
                                    _ => Color::White,
                                };

                                let prefix = if is_selected { "> " } else { "- " };

                                // Component Name Line
                                lines.push(Line::from(vec![
                                    Span::styled(prefix, style),
                                    Span::styled(
                                        comp.component_name
                                            .as_deref()
                                            .unwrap_or("Unknown Component"),
                                        style.add_modifier(Modifier::BOLD),
                                    ),
                                    Span::raw(": "),
                                    Span::styled(
                                        comp.component_status.as_deref().unwrap_or("N/A"),
                                        if is_selected {
                                            style
                                        } else {
                                            Style::default().fg(status_color)
                                        },
                                    ),
                                ]));

                                // Warnings (indented)
                                if let Some(warnings) = comp.number_of_warnings {
                                    if warnings > 0 {
                                        lines.push(Line::from(vec![
                                            Span::raw("    Warnings: "),
                                            Span::styled(
                                                warnings.to_string(),
                                                Style::default().fg(Color::Yellow),
                                            ),
                                        ]));
                                    }
                                }
                            }
                        }
                        JobViewRow::StdOutLink(_) => {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                Span::styled(
                                    "View Standard Output",
                                    if is_selected {
                                        style.fg(Color::Cyan)
                                    } else {
                                        Style::default().fg(Color::Cyan)
                                    },
                                ),
                            ]));
                        }
                        JobViewRow::StdErrLink(_) => {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                Span::styled(
                                    "View Standard Error",
                                    if is_selected {
                                        style.fg(Color::Red)
                                    } else {
                                        Style::default().fg(Color::Red)
                                    },
                                ),
                            ]));
                        }
                    }
                }
            } else {
                lines.push(Line::from("No components found."));
            }
        } else {
            // Only show this if we aren't loading and don't have a result yet (e.g. no job UID found)
            lines.push(Line::from(Span::styled(
                "No Job Result information available.",
                Style::default().fg(Color::Gray),
            )));
        }

        let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
        frame.render_widget(p, area);
    } else {
        frame.render_widget(
            Paragraph::new("No activity log selected")
                .block(Block::default().borders(Borders::ALL)),
            area,
        );
    }
}

fn render_popup(app: &App, frame: &mut Frame) {
    if app.show_popup {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(app.popup_title.as_str());
        let area = centered_rect(60, 60, frame.area());

        frame.render_widget(Clear, area); // Clear the area below the popup

        if app.popup_loading {
            frame.render_widget(
                Paragraph::new("Loading...")
                    .block(block)
                    .alignment(Alignment::Center),
                area,
            );
        } else {
            let p = Paragraph::new(app.popup_content.as_str())
                .block(block)
                .wrap(Wrap { trim: true })
                .scroll((0, 0)); // TODO: Add scrolling state for popup if content is long
            frame.render_widget(p, area);
        }
    }
}

fn render_device_search_popup(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(80, 60, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Search Devices ")
        .title_bottom(Line::from(" Esc: close | Enter: select ").right_aligned())
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(block.clone(), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Length(1), // Status/Warning
            Constraint::Min(0),    // Results
        ])
        .split(area);

    // Input
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(" Hostname Search ")
        .border_style(Style::default().fg(Color::Cyan));

    let input = Paragraph::new(app.device_search_query.clone())
        .block(input_block)
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(input, layout[0]);

    // Status/Warning
    let status_text = if app.device_search_loading {
        Span::styled("Loading...", Style::default().fg(Color::Yellow))
    } else if let Some(err) = &app.device_search_error {
        Span::styled(format!("Error: {}", err), Style::default().fg(Color::Red))
    } else if app.device_search_query.len() < 3 {
        Span::styled(
            "Type at least 3 characters...",
            Style::default().fg(Color::Gray),
        )
    } else if app.device_search_results.is_empty() && !app.device_search_query.is_empty() {
        Span::styled("No results found.", Style::default().fg(Color::Yellow))
    } else {
        Span::styled(
            format!("Found {} devices", app.device_search_results.len()),
            Style::default().fg(Color::Green),
        )
    };

    frame.render_widget(Paragraph::new(status_text), layout[1]);

    // Results
    if !app.device_search_results.is_empty() {
        let rows: Vec<Row> = app
            .device_search_results
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let style = if Some(i) == app.device_search_table_state.selected() {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                let status = if d.online { "Online" } else { "Offline" };
                let status_color = if d.online { Color::Green } else { Color::Gray };

                let os = d.operating_system.as_deref().unwrap_or("N/A");
                let patch = d
                    .patch_management
                    .as_ref()
                    .and_then(|pm| pm.patch_status.clone())
                    .unwrap_or("Unknown".to_string());

                Row::new(vec![
                    Cell::from(d.hostname.clone()),
                    Cell::from(d.site_name.as_deref().unwrap_or("").to_string()),
                    Cell::from(Span::styled(status, Style::default().fg(status_color))),
                    Cell::from(os),
                    Cell::from(patch),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25), // Hostname
                Constraint::Percentage(25), // Site
                Constraint::Percentage(10), // Status
                Constraint::Percentage(25), // OS
                Constraint::Percentage(15), // Patch
            ],
        )
        .header(
            Row::new(vec!["Hostname", "Site", "Status", "OS", "Patch"]).style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Results ")
                .border_style(Style::default().fg(Color::White)),
        )
        .highlight_symbol(">> ");

        frame.render_stateful_widget(table, layout[2], &mut app.device_search_table_state);
    }
}
