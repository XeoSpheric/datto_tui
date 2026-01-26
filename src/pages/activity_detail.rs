use crate::app::{App, JobViewRow};
use crate::common::jobs::generate_job_rows;
use crate::common::utils::format_timestamp;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render_activity_detail(app: &mut App, frame: &mut Frame, area: Rect) {
    if let Some(log) = &app.selected_activity_log {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Activity Log Details");

        // Format date
        let date_str = format_timestamp(log.date.map(serde_json::Value::from));

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
            let ran_on_str = format_timestamp(job_result.ran_on.clone());

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
