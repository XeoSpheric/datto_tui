use crate::app::{App, InputField, QuickAction, RebootFocus, RunComponentStep};
use crate::common::utils::centered_rect;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};

pub fn render_input_modal(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area); // Clear background

    let (title, is_single_field_edit) = if let Some(field) = &app.input_state.editing_setting {
        (format!("Edit Setting: {:?}", field), true)
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
        // Variable Edit
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

pub fn render_quick_action_menu(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(40, 30, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Quick Actions (Esc to cancel)")
        .style(Style::default().bg(Color::DarkGray));

    let rows: Vec<Row> = app
        .quick_actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let style = if Some(i) == app.quick_action_list_state.selected() {
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::Yellow)
            } else {
                Style::default()
            };

            let label = match action {
                QuickAction::ScheduleReboot => "Schedule Reboot",
                QuickAction::RunComponent => "Run Component",
                QuickAction::RunAvScan => "Run AV Scan",
                QuickAction::OpenWebRemote => "Open Web Remote",
                QuickAction::ReloadData => "Reload Data",
                QuickAction::MoveToSite => "Move Device to Site",
                QuickAction::UpdateWarranty => "Update Warranty",
                QuickAction::ClearWarranty => "Clear Warranty",
            };

            Row::new(vec![Cell::from(label)]).style(style)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Percentage(100)])
        .block(block)
        .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut app.quick_action_list_state);
}

pub fn render_warranty_popup(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Update Warranty Date (YYYY-MM-DD)")
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(block.clone(), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Input Row
            Constraint::Length(1), // Error
            Constraint::Min(0),    // Instructions
        ])
        .split(block.inner(area));

    let input_area = layout[0];
    let segments_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // YYYY
            Constraint::Length(7),  // MM
            Constraint::Length(7),  // DD
        ])
        .split(input_area);

    let labels = ["YYYY", "MM", "DD"];
    let focuses = [
        crate::app::WarrantyFocus::Year,
        crate::app::WarrantyFocus::Month,
        crate::app::WarrantyFocus::Day,
    ];

    for i in 0..3 {
        let style = if app.warranty_focus == focuses[i] {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(labels[i])
            .style(style);
        let p = Paragraph::new(app.warranty_segments[i].clone())
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(p, segments_layout[i]);
    }

    if let Some(err) = &app.warranty_error {
        let err_p = Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red));
        frame.render_widget(err_p, layout[1]);
    }

    let instructions = Paragraph::new("Tab: Switch | Enter: Submit | x: Clear All | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::ITALIC));
    frame.render_widget(instructions, layout[2]);
}

pub fn render_reboot_popup(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(50, 40, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Schedule Reboot")
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(block.clone(), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Reboot Now
            Constraint::Length(3), // Reboot Time
            Constraint::Length(1), // Error
            Constraint::Min(0),    // Instructions
        ])
        .split(block.inner(area));

    // Reboot Now Checkbox
    let now_style = if app.reboot_focus == RebootFocus::RebootNow {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let now_text = if app.reboot_now {
        "[x] Reboot Now"
    } else {
        "[ ] Reboot Now"
    };
    let now_p = Paragraph::new(now_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Action")
            .style(now_style),
    );
    frame.render_widget(now_p, layout[0]);

    // Reboot Time Input
    let time_area = layout[1];
    let segments_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(7), // YY
            Constraint::Length(7), // MM
            Constraint::Length(7), // DD
            Constraint::Length(7), // HH
            Constraint::Length(7), // mm
        ])
        .split(time_area);

    let labels = ["YY", "MM", "DD", "HH", "mm"];
    let focuses = [
        RebootFocus::Year,
        RebootFocus::Month,
        RebootFocus::Day,
        RebootFocus::Hour,
        RebootFocus::Minute,
    ];

    for i in 0..5 {
        let style = if app.reboot_focus == focuses[i] {
            Style::default().fg(Color::Yellow)
        } else if app.reboot_now {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(labels[i])
            .style(style);
        let p = Paragraph::new(app.reboot_segments[i].clone())
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(p, segments_layout[i]);
    }

    // Error Message
    if let Some(err) = &app.reboot_error {
        let err_p = Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red));
        frame.render_widget(err_p, layout[2]);
    }

    // Instructions
    let instructions = Paragraph::new("Space: Toggle | Tab: Switch | Enter: Submit | Esc: Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::ITALIC));
    frame.render_widget(instructions, layout[3]);
}

pub fn render_run_component_popup(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    let title = match app.run_component_step {
        RunComponentStep::Search => "Run Component - Select (Esc to cancel)",
        RunComponentStep::FillVariables => "Run Component - Variables (Esc to back)",
        RunComponentStep::Review => "Run Component - Review (Esc to back, Enter to Run)",
        RunComponentStep::Result => "Run Component - Result (Enter/Esc to close)",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(block.clone(), area);

    let inner_area = block.inner(area);

    match app.run_component_step {
        RunComponentStep::Search => {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Search Input
                    Constraint::Min(0),    // List
                ])
                .split(inner_area);

            // Search Input
            let input_block = Block::default()
                .borders(Borders::ALL)
                .title("Search Component");
            let input = Paragraph::new(app.component_search_query.clone()).block(input_block);
            frame.render_widget(input, layout[0]);

            // Component List
            if app.components_loading {
                frame.render_widget(
                    Paragraph::new("Loading components...").alignment(Alignment::Center),
                    layout[1],
                );
            } else if let Some(err) = &app.component_error {
                frame.render_widget(
                    Paragraph::new(format!("Error: {}", err))
                        .style(Style::default().fg(Color::Red)),
                    layout[1],
                );
            } else {
                let rows: Vec<Row> = app
                    .filtered_components
                    .iter()
                    .enumerate()
                    .map(|(i, comp)| {
                        let style = if Some(i) == app.component_list_state.selected() {
                            Style::default().add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        };
                        Row::new(vec![
                            Cell::from(comp.name.clone()),
                            Cell::from(comp.category_code.clone().unwrap_or_default()),
                            Cell::from(comp.description.clone().unwrap_or_default()),
                        ])
                        .style(style)
                    })
                    .collect();

                let table = Table::new(
                    rows,
                    [
                        Constraint::Percentage(30),
                        Constraint::Percentage(15),
                        Constraint::Percentage(55),
                    ],
                )
                .header(
                    Row::new(vec!["Name", "Category", "Description"])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .highlight_symbol(">> ");

                frame.render_stateful_widget(table, layout[1], &mut app.component_list_state);
            }
        }
        RunComponentStep::FillVariables => {
            if let Some(component) = &app.selected_component {
                if let Some(vars) = &component.variables {
                    if let Some(current_var) =
                        app.component_variables.get(app.component_variable_index)
                    {
                        let def = vars.iter().find(|v| v.name == current_var.name);

                        let layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Length(3), // Progress
                                Constraint::Length(5), // Variable Info
                                Constraint::Length(3), // Input
                                Constraint::Min(0),    // Description/Help
                            ])
                            .split(inner_area);

                        let progress = format!(
                            "Variable {} of {}",
                            app.component_variable_index + 1,
                            app.component_variables.len()
                        );
                        frame.render_widget(
                            Paragraph::new(progress).alignment(Alignment::Center),
                            layout[0],
                        );

                        let mut type_str = def
                            .and_then(|v| v.variable_type.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("String")
                            .to_string();

                        if type_str.eq_ignore_ascii_case("map") {
                            type_str = "Selection (Map)".to_string();
                        }

                        let info_text = vec![
                            Line::from(vec![
                                Span::styled(
                                    "Name: ",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(&current_var.name),
                            ]),
                            Line::from(vec![
                                Span::styled(
                                    "Type: ",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(type_str),
                            ]),
                        ];
                        frame.render_widget(Paragraph::new(info_text), layout[1]);

                        let input_block = Block::default()
                            .borders(Borders::ALL)
                            .title("Value")
                            .style(Style::default().fg(Color::Yellow));

                        let input_val = app.component_variable_input.clone();

                        frame
                            .render_widget(Paragraph::new(input_val).block(input_block), layout[2]);

                        if let Some(d) = def {
                            if let Some(desc) = &d.description {
                                let desc_block =
                                    Block::default().borders(Borders::ALL).title("Description");
                                frame.render_widget(
                                    Paragraph::new(desc.as_str())
                                        .block(desc_block)
                                        .wrap(Wrap { trim: true }),
                                    layout[3],
                                );
                            }
                        }
                    }
                }
            }
        }
        RunComponentStep::Review => {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Variables List
                    Constraint::Length(3), // Footer
                ])
                .split(inner_area);

            if let Some(comp) = &app.selected_component {
                frame.render_widget(
                    Paragraph::new(format!("Review Job: {}", comp.name))
                        .style(
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Cyan),
                        )
                        .alignment(Alignment::Center),
                    layout[0],
                );

                let rows: Vec<Row> = app
                    .component_variables
                    .iter()
                    .map(|v| {
                        Row::new(vec![
                            Cell::from(v.name.clone()),
                            Cell::from(v.value.clone()),
                        ])
                    })
                    .collect();

                let table = Table::new(
                    rows,
                    [Constraint::Percentage(40), Constraint::Percentage(60)],
                )
                .header(
                    Row::new(vec!["Variable", "Value"])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(Block::default().borders(Borders::ALL));

                frame.render_widget(table, layout[1]);

                frame.render_widget(
                    Paragraph::new("Press ENTER to Execute Job")
                        .style(
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::SLOW_BLINK),
                        )
                        .alignment(Alignment::Center),
                    layout[2],
                );
            }
        }
        RunComponentStep::Result => {
            if app.components_loading {
                frame.render_widget(
                    Paragraph::new("Executing Job...").alignment(Alignment::Center),
                    inner_area,
                );
            } else if let Some(err) = &app.component_error {
                frame.render_widget(
                    Paragraph::new(format!("Error: {}", err))
                        .style(Style::default().fg(Color::Red))
                        .wrap(Wrap { trim: true }),
                    inner_area,
                );
            } else if let Some(response) = &app.last_job_response {
                let job_info = response.job.as_ref();
                let job_name = job_info
                    .and_then(|j| j.name.as_deref())
                    .unwrap_or("Unknown");
                let job_id = job_info
                    .map(|j| j.id.to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                let job_status = job_info
                    .and_then(|j| j.status.as_deref())
                    .unwrap_or("Unknown");

                let text = vec![
                    Line::from(Span::styled(
                        "Job Executed Successfully!",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Job Name: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(job_name),
                    ]),
                    Line::from(vec![
                        Span::styled("Job ID: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(job_id),
                    ]),
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(job_status),
                    ]),
                    Line::from(""),
                    Line::from("Check Activity Log for status."),
                ];
                frame.render_widget(
                    Paragraph::new(text).alignment(Alignment::Center),
                    inner_area,
                );
            }
        }
    }
}

pub fn render_popup(app: &App, frame: &mut Frame) {
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
                .scroll((0, 0));
            frame.render_widget(p, area);
        }
    }
}

pub fn render_device_search_popup(app: &mut App, frame: &mut Frame) {
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

pub fn render_device_variables_popup(
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

pub fn render_site_move_popup(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Move Device to Site ")
        .title_bottom(Line::from(" Esc: cancel | Enter: move ").right_aligned())
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(block.clone(), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Search Input
            Constraint::Min(0),    // Site List
        ])
        .split(area);

    // Search Input
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(" Filter Sites ")
        .border_style(Style::default().fg(Color::Cyan));
    let input = Paragraph::new(app.site_move_query.clone()).block(input_block);
    frame.render_widget(input, layout[0]);

    // Results
    let rows: Vec<Row> = app
        .filtered_sites
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if Some(i) == app.site_move_table_state.selected() {
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::Yellow)
            } else {
                Style::default()
            };
            Row::new(vec![Cell::from(s.name.clone())]).style(style)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Percentage(100)]).highlight_symbol(">> ");

    frame.render_stateful_widget(table, layout[1], &mut app.site_move_table_state);
}
