use crate::app::{App, CurrentView, InputField, InputMode, SiteDetailTab, SiteEditField};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs, Wrap},
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
                "Datto TUI | Sites: {} | Page {}/{} | 'q': quit, 'r': reload, 'j/k': move, 'n/p': page, 'Enter': details",
                app.total_count,
                app.current_page + 1,
                if app.total_pages == 0 {
                    1
                } else {
                    app.total_pages
                }
            )
        }
        CurrentView::Detail => "Detail View | 'Esc'/'q': back".to_string(),
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
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
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

    let (title, is_settings_edit) = if let Some(field) = &app.input_state.editing_setting {
        (format!("Edit Setting: {:?}", field), true) // Primitive debug format for now, can perform better mapping later
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

    // If it's a settings edit, we only need one input field. If variable, we might need two (Name/Value)
    let constraints = if is_settings_edit {
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

    if is_settings_edit {
        // Single input field for settings
        let input_style = Style::default().fg(Color::Yellow);
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("Value")
            .style(input_style);
        let input_text = Paragraph::new(app.input_state.name_buffer.clone()).block(input_block); // We reused name_buffer for single field
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
