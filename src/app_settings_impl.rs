fn next_setting(&mut self) {
    let i = match self.settings_table_state.selected() {
        Some(i) => {
            if i >= 4 {
                // 5 items: Name, Desc, Notes, OnDemand, Splashtop (0-4)
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    self.settings_table_state.select(Some(i));
}

fn prev_setting(&mut self) {
    let i = match self.settings_table_state.selected() {
        Some(i) => {
            if i == 0 {
                4
            } else {
                i - 1
            }
        }
        None => 0,
    };
    self.settings_table_state.select(Some(i));
}

fn open_edit_setting_modal(&mut self) {
    // Ensure site edit state is fresh
    // self.populate_site_edit_state(); // This is called on tab switch, should be fine.

    // Determine which setting is selected
    let setting_idx = self.settings_table_state.selected().unwrap_or(0);
    let (field_type, current_value) = match setting_idx {
        0 => (SiteEditField::Name, self.site_edit_state.name.clone()),
        1 => (
            SiteEditField::Description,
            self.site_edit_state.description.clone(),
        ),
        2 => (SiteEditField::Notes, self.site_edit_state.notes.clone()),
        // boolean fields technically "edit" via toggle, but could support text input "true"/"false" if desired.
        // For now, let's only support Editing Modal for the text fields.
        // Bools are handled by Space/Enter toggle.
        _ => return,
    };

    let active_input = match setting_idx {
        0 => InputField::SiteName,
        1 => InputField::SiteDescription,
        2 => InputField::SiteNotes,
        _ => InputField::Name, // Fallback
    };

    self.input_state = InputState {
        mode: InputMode::Editing,
        name_buffer: current_value, // Re-use name_buffer for the single value being edited
        value_buffer: String::new(), // Not used for single-value setting edit
        active_field: active_input, // Tells us which field on the SiteEditState to update on submit
        is_creating: false,
        editing_variable_id: None,
        editing_setting: Some(field_type),
    };
}

fn toggle_setting(&mut self, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
    let setting_idx = self.settings_table_state.selected().unwrap_or(0);
    match setting_idx {
        3 => {
            // On Demand
            self.site_edit_state.on_demand = !self.site_edit_state.on_demand;
            self.submit_site_update(tx);
        }
        4 => {
            // Splashtop
            self.site_edit_state.splashtop_auto_install =
                !self.site_edit_state.splashtop_auto_install;
            self.submit_site_update(tx);
        }
        _ => {
            // If it's a text field, Enter also behaves like 'e' -> Open Edit
            self.open_edit_setting_modal();
        }
    }
}
