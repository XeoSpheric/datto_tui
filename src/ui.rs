use crate::app::{App, CurrentView};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
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
                "Datto TUI | Sites: {} | Page {}/{} | 'q': quit, 'j/k': move, 'n/p': page, 'Enter': details",
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

            Row::new(vec![
                Cell::from(site.name.clone()),
                Cell::from(device_count.to_string()),
                Cell::from(site.uid.clone()),
            ])
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
        Row::new(vec!["Site Name", "Devices", "UID"])
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

    // --- Right Pane: Device List ---
    let devices_block = Block::default().borders(Borders::ALL).title("Devices");

    if app.devices_loading {
        frame.render_widget(
            Paragraph::new("Loading devices...").block(devices_block),
            chunks[1],
        );
    } else if let Some(err) = &app.devices_error {
        frame.render_widget(
            Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(Color::Red))
                .block(devices_block),
            chunks[1],
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

                Row::new(vec![
                    Cell::from(device.hostname.clone()),
                    Cell::from(Span::styled(status, Style::default().fg(status_color))),
                    Cell::from(device.operating_system.clone().unwrap_or_default()),
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
            Row::new(vec!["Hostname", "Status", "OS"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(devices_block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(table, chunks[1], &mut app.devices_table_state);
    }
}
