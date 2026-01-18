use crate::app::{App, CurrentView, DeviceDetailTab, InputField, InputMode, SiteDetailTab};
use chrono::{DateTime, Local};
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
                "Kyber TUI | Sites: {} | 'q': quit, 'r': reload, 'j/k': move, 'Enter': details",
                app.total_count
            )
        }
        CurrentView::Detail => "Detail View | 'Esc'/'q': back".to_string(),
        CurrentView::DeviceDetail => "Device Detail | 'Esc'/'q': back".to_string(),
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
        }
    }

    // Render Input Modal if Editing
    if app.input_state.mode == InputMode::Editing {
        render_input_modal(app, frame);
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
    if let Some(device) = &app.selected_device {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // --- Left Pane: Device Info ---
        render_device_info(device, frame, chunks[0]);

        // --- Right Pane: Tabs & Content ---
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(chunks[1]);

        let tabs = Tabs::new(vec!["Variables", "Security", "Jobs"])
            .select(match app.device_detail_tab {
                DeviceDetailTab::Variables => 0,
                DeviceDetailTab::Security => 1,
                DeviceDetailTab::Jobs => 2,
            })
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            );
        frame.render_widget(tabs, right_chunks[0]);

        match app.device_detail_tab {
            DeviceDetailTab::Variables => {
                render_device_variables(device, frame, right_chunks[1], &mut app.udf_table_state)
            }
            DeviceDetailTab::Security => {
                render_device_security(app, device, frame, right_chunks[1])
            }
            DeviceDetailTab::Jobs => render_device_jobs(device, frame, right_chunks[1]),
        }
    } else {
        frame.render_widget(
            Paragraph::new("No device selected").block(Block::default().borders(Borders::ALL)),
            area,
        );
    }
}

fn render_device_info(device: &crate::api::datto::types::Device, frame: &mut Frame, area: Rect) {
    // Format Dates
    let last_seen_str = format_timestamp(device.last_seen.clone());
    let last_reboot_str = format_timestamp(device.last_reboot.clone());
    let last_audit_str = format_timestamp(device.last_audit_date.clone());
    let creation_date_str = format_timestamp(device.creation_date.clone());

    let text = vec![
        Line::from(vec![
            Span::styled("Hostname: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&device.hostname),
        ]),
        Line::from(vec![
            Span::styled("UID: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&device.uid),
        ]),
        Line::from(vec![
            Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.id.to_string()),
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
            Span::styled(
                "Display Version: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(device.display_version.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(vec![
            Span::styled("Last User: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.last_logged_in_user.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                if device.online { "Online" } else { "Offline" },
                if device.online {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("Int. IP: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.int_ip_address.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(vec![
            Span::styled("Ext. IP: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.ext_ip_address.as_deref().unwrap_or("N/A")),
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
            Span::raw(device.warranty_date.as_deref().unwrap_or("N/A")),
        ]),
        Line::from(vec![
            Span::styled(
                "Patch Status: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                device
                    .patch_management
                    .as_ref()
                    .and_then(|pm| pm.patch_status.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
            ),
        ]),
    ];

    let info_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Device Info: {}", device.hostname));

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

fn render_device_variables(
    device: &crate::api::datto::types::Device,
    frame: &mut Frame,
    area: Rect,
    state: &mut TableState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("User Defined Fields (UDF) - Press 'Enter' to Edit");

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

fn render_device_security(
    app: &App,
    device: &crate::api::datto::types::Device,
    frame: &mut Frame,
    area: Rect,
) {
    let block = Block::default().borders(Borders::ALL).title("Security");

    let mut text = vec![
        Line::from(vec![
            Span::styled(
                "Anti-Virus Product: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                device
                    .antivirus
                    .as_ref()
                    .and_then(|av| av.antivirus_product.clone())
                    .unwrap_or("N/A".to_string()),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Anti-Virus Status: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                device
                    .antivirus
                    .as_ref()
                    .and_then(|av| av.antivirus_status.clone())
                    .unwrap_or("N/A".to_string()),
            ),
        ]),
    ];

    // Sophos Integration
    let loading = app
        .sophos_loading
        .get(&device.hostname)
        .copied()
        .unwrap_or(false);

    if loading {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            "Loading Sophos data...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )));
    } else if let Some(endpoint) = app.sophos_endpoints.get(&device.hostname) {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            "Sophos Endpoint Details",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )));

        let health_status = endpoint
            .health
            .as_ref()
            .and_then(|h| h.overall.clone())
            .unwrap_or("Unknown".to_string());

        let health_color = match health_status.to_lowercase().as_str() {
            "good" => Color::Green,
            "bad" | "critical" => Color::Red,
            "suspicious" => Color::Yellow,
            _ => Color::White,
        };

        text.push(Line::from(vec![
            Span::styled("Health: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(health_status, Style::default().fg(health_color)),
        ]));

        let isolated = endpoint
            .isolation
            .as_ref()
            .and_then(|i| i.is_isolated)
            .unwrap_or(false);

        text.push(Line::from(vec![
            Span::styled("Isolation: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                if isolated { "ISOLATED" } else { "Not Isolated" },
                if isolated {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
        ]));

        text.push(Line::from(""));

        // Scan Status
        // Scan Status
        let scan_status = app.scan_status.get(&device.hostname);

        match scan_status {
            Some(crate::event::ScanStatus::Starting) => {
                text.push(Line::from(Span::styled(
                    "Scan starting...",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            Some(crate::event::ScanStatus::Started) => {
                text.push(Line::from(Span::styled(
                    "Scan Started",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            None => {
                text.push(Line::from(Span::styled(
                    "Press 's' to start scan",
                    Style::default().add_modifier(Modifier::ITALIC),
                )));
            }
        }
    }

    // Datto AV Integration
    let datto_loading = app
        .datto_av_loading
        .get(&device.hostname)
        .copied()
        .unwrap_or(false);

    if datto_loading {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            "Loading Datto AV data...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )));
    } else if let Some(agent) = app.datto_av_agents.get(&device.hostname) {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            "Datto AV Agent Details",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )));

        text.push(Line::from(vec![
            Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(agent.version.as_deref().unwrap_or("N/A")),
        ]));

        text.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(agent.status.as_deref().unwrap_or("N/A")),
        ]));

        let isolated = agent.isolated.unwrap_or(false);
        text.push(Line::from(vec![
            Span::styled("Isolation: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                if isolated { "ISOLATED" } else { "Not Isolated" },
                if isolated {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
        ]));

        if let Some(hb) = &agent.heartbeat {
            text.push(Line::from(vec![
                Span::styled("Heartbeat: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(hb),
            ]));
        }

        text.push(Line::from(""));

        // Scan Status
        // Scan Status
        let scan_status = app.scan_status.get(&device.hostname);

        match scan_status {
            Some(crate::event::ScanStatus::Starting) => {
                text.push(Line::from(Span::styled(
                    "Scan starting...",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            Some(crate::event::ScanStatus::Started) => {
                text.push(Line::from(Span::styled(
                    "Scan Started",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            None => {
                text.push(Line::from(Span::styled(
                    "Press 's' to start scan",
                    Style::default().add_modifier(Modifier::ITALIC),
                )));
            }
        }
    }

    // Alerts Section
    if let Some(alerts) = app.datto_av_alerts.get(&device.hostname) {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            "5 Most Recent Alerts",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )));

        if alerts.is_empty() {
            text.push(Line::from(Span::styled(
                "No recent alerts",
                Style::default().fg(Color::Green),
            )));
        } else {
            for alert in alerts.iter().take(5) {
                let name = alert.name.as_deref().unwrap_or("Unknown Alert");
                let severity = alert.severity.as_deref().unwrap_or("Unknown");
                let date_str = alert.created_on.as_deref().unwrap_or("");

                let formatted_date = if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
                    dt.with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                } else {
                    date_str.to_string()
                };

                let severity_style = match severity.to_lowercase().as_str() {
                    "critical" | "high" => Style::default().fg(Color::Red),
                    "medium" => Style::default().fg(Color::Yellow),
                    _ => Style::default().fg(Color::White),
                };

                text.push(Line::from(vec![
                    Span::styled(format!("• {} ", name), Style::default()),
                    Span::styled(format!("({}) ", severity), severity_style),
                    Span::styled(formatted_date, Style::default().add_modifier(Modifier::DIM)),
                ]));
            }
        }
    } else {
        // If alerts haven't been fetched yet or not Datto AV
        // text.push(Line::from(""));
        // text.push(Line::from(Span::styled(
        //     "Alerts loading...",
        //     Style::default().add_modifier(Modifier::ITALIC),
        // )));
    }

    let p = Paragraph::new(text).block(block);
    frame.render_widget(p, area);
}

fn render_device_jobs(_device: &crate::api::datto::types::Device, frame: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Jobs");
    let p = Paragraph::new("Jobs functionality coming soon...").block(block);
    frame.render_widget(p, area);
}
