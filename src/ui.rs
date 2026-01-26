use crate::app::{App, CurrentView, InputMode};
use crate::pages::{
    activity_detail::render_activity_detail,
    device_detail::render_device_detail,
    popups::{
        render_device_search_popup, render_input_modal, render_popup, render_quick_action_menu,
        render_reboot_popup, render_run_component_popup, render_site_move_popup,
        render_warranty_popup,
    },
    site_detail::render_site_detail,
    site_list::render_site_list,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
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
        CurrentView::Detail => {
            "Site Detail View | 'Esc'/'q': back, '/': search, 'Space': select, 'r': quick actions"
                .to_string()
        }
        CurrentView::DeviceDetail => {
            "Device Detail | 'Esc'/'q': back, 'r': quick actions, 'v': variables".to_string()
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
            Paragraph::new("Loading...")
                .style(Style::default().fg(Color::Yellow))
                .block(main_block),
            layout[1],
        );
    } else {
        match app.current_view {
            CurrentView::List => render_site_list(app, frame, layout[1], main_block),
            CurrentView::Detail => render_site_detail(app, frame, layout[1]),
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

    // Render Run Component Popup
    if app.show_run_component {
        render_run_component_popup(app, frame);
    }

    // Render Quick Actions Menu
    if app.show_quick_actions {
        render_quick_action_menu(app, frame);
    }

    // Render Reboot Popup
    if app.show_reboot_popup {
        render_reboot_popup(app, frame);
    }

    // Render Site Move Popup
    if app.show_site_move {
        render_site_move_popup(app, frame);
    }

    // Render Warranty Popup
    if app.show_warranty_popup {
        render_warranty_popup(app, frame);
    }
}
