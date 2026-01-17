use crate::api::datto::DattoClient;
use crate::api::datto::types::{
    CreateVariableRequest, Device, Site, UpdateSiteRequest, UpdateVariableRequest,
};
use crate::event::{Event, EventHandler};
use crate::tui::Tui;
use crate::ui;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;

use crate::api::rocket_cyber::RocketCyberClient;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct IncidentStats {
    pub active: i32,
    pub resolved: i32,
}

#[derive(Debug, PartialEq)]
pub enum CurrentView {
    List,
    Detail,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SiteDetailTab {
    Devices,
    Variables,
    Settings,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SiteEditField {
    Name,
    Description,
    Notes,
}

#[derive(Debug)]
pub struct SiteEditState {
    pub name: String,
    pub description: String,
    pub notes: String,
    pub on_demand: bool,
    pub splashtop_auto_install: bool,
    pub active_field: SiteEditField,
    pub is_editing: bool, // Track if we are in "edit mode" for settings (or just viewing) - simplification: settings is always editable input fields
}

impl Default for SiteEditState {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            notes: String::new(),
            on_demand: false,
            splashtop_auto_install: false,
            active_field: SiteEditField::Name,
            is_editing: false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputField {
    Name,
    Value,
    // Add fields for Site Settings
    SiteName,
    SiteDescription,
    SiteNotes,
}

#[derive(Debug)]
pub struct InputState {
    pub mode: InputMode,
    pub name_buffer: String,
    pub value_buffer: String,
    pub active_field: InputField,
    pub is_creating: bool, // true = create, false = update
    pub editing_variable_id: Option<i32>,
    // Add context for what we are editing if not a variable
    pub editing_setting: Option<SiteEditField>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mode: InputMode::Normal,
            name_buffer: String::new(),
            value_buffer: String::new(),
            active_field: InputField::Name,
            is_creating: true,
            editing_variable_id: None,
            editing_setting: None,
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub counter: u8,
    // Sites
    pub sites: Vec<Site>,
    // RocketCyber Incidents
    pub incidents: Vec<crate::api::rocket_cyber::types::Incident>,
    // Aggregated Stats: Key is lowercased account name
    pub incident_stats: HashMap<String, IncidentStats>,

    pub is_loading: bool,
    pub error: Option<String>,
    pub client: Option<DattoClient>,
    pub rocket_client: Option<RocketCyberClient>, // Add field
    pub current_view: CurrentView,

    // Navigation & Pagination (Sites)
    pub table_state: TableState,
    pub current_page: i32,
    pub total_pages: i32,
    pub total_count: i32,

    // Devices
    pub devices: Vec<Device>,
    pub devices_loading: bool,
    pub devices_error: Option<String>,
    pub devices_table_state: TableState,
    pub detail_tab: SiteDetailTab,

    // Input
    pub input_state: InputState,
    pub variables_table_state: TableState,

    // Site Edit
    pub site_edit_state: SiteEditState,
    pub settings_table_state: TableState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            counter: 0,
            sites: Vec::new(),
            incidents: Vec::new(),
            incident_stats: HashMap::new(),
            is_loading: false,
            error: None,
            client: None,
            rocket_client: None, // Initialize
            current_view: CurrentView::List,

            table_state: TableState::default(),
            current_page: 0,
            total_pages: 0,
            total_count: 0,

            devices: Vec::new(),
            devices_loading: false,
            devices_error: None,
            devices_table_state: TableState::default(),
            detail_tab: SiteDetailTab::Devices,
            input_state: InputState::default(),
            variables_table_state: TableState::default(),
            site_edit_state: SiteEditState::default(),
            settings_table_state: TableState::default(),
        }
    }
}

impl App {
    pub fn new(client: Option<DattoClient>, rocket_client: Option<RocketCyberClient>) -> Self {
        let mut app = Self::default();
        app.client = client;
        app.rocket_client = rocket_client;
        app
    }

    pub async fn run(&mut self, tui: &mut Tui, events: &mut EventHandler) -> Result<()> {
        // Initial fetch
        if self.client.is_some() {
            self.fetch_sites(events.sender());
        } else {
            self.error = Some("API Client not initialized. Check .env config.".to_string());
        }

        // Fetch incidents
        if self.rocket_client.is_some() {
            self.fetch_rocket_incidents(events.sender());
        }

        while !self.should_quit {
            tui.draw(|f| {
                ui::render(self, f);
            })?;

            match events.next().await? {
                Event::Tick => {}
                Event::Key(key) => self.handle_key_event(key, events.sender()),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                Event::SitesFetched(result) => {
                    self.is_loading = false;
                    match result {
                        Ok(mut response) => {
                            // Sort sites alphabetically by name
                            response
                                .sites
                                .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                            self.sites = response.sites;

                            // Update pagination info
                            self.total_count = response.page_details.total_count.unwrap_or(0);
                            // Calculate total pages (assuming max=50)
                            if self.total_count > 0 {
                                self.total_pages = (self.total_count as f64 / 50.0).ceil() as i32;
                            } else {
                                self.total_pages = 1;
                            }

                            if !self.sites.is_empty() {
                                self.table_state.select(Some(0));
                                // Fetch variables for all sites on this page
                                for site in &self.sites {
                                    self.fetch_site_variables(site.uid.clone(), events.sender());
                                }
                            } else {
                                self.table_state.select(None);
                            }
                        }
                        Err(e) => {
                            self.error = Some(e.to_string());
                        }
                    }
                }
                Event::DevicesFetched(result) => {
                    self.devices_loading = false;
                    match result {
                        Ok(response) => {
                            self.devices = response.devices;
                            if !self.devices.is_empty() {
                                self.devices_table_state.select(Some(0));
                            } else {
                                self.devices_table_state.select(None);
                            }
                        }
                        Err(e) => {
                            self.devices_error = Some(e.to_string());
                        }
                    }
                }
                Event::IncidentsFetched(result) => match result {
                    Ok(incidents) => {
                        self.incidents = incidents;
                        // Aggregate stats
                        self.incident_stats.clear();
                        for incident in &self.incidents {
                            // Normalize name for matching: lowercase and trim
                            let account_name = incident.account_name.to_lowercase();
                            // This is a naive match key. In reality we might need a better mapping.
                            // However Datto site names and RocketCyber account names are "close".
                            // For now we will use the lowercase name from RocketCyber as the key.
                            // When looking up from Datto Site, we will also lowercase that name.

                            let entry = self
                                .incident_stats
                                .entry(account_name)
                                .or_insert(IncidentStats::default());

                            // Check status
                            let status = incident.status.to_lowercase();
                            if status == "resolved" {
                                entry.resolved += 1;
                            } else {
                                entry.active += 1;
                            }

                            // Also index by Account ID for variable mapping
                            let account_id = incident.account_id.to_string();
                            let entry_id = self
                                .incident_stats
                                .entry(account_id)
                                .or_insert(IncidentStats::default());

                            if status == "resolved" {
                                entry_id.resolved += 1;
                            } else {
                                entry_id.active += 1;
                            }
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to fetch incidents: {}", e));
                    }
                },
                Event::SiteVariablesFetched(site_uid, result) => match result {
                    Ok(variables) => {
                        if let Some(site) = self.sites.iter_mut().find(|s| s.uid == site_uid) {
                            site.variables = Some(variables);
                        }
                    }
                    Err(_e) => {
                        // Log error or ignore? For now, maybe just print to stderr if debug
                        // self.error = Some(format!("Failed to fetch variables for {}: {}", site_uid, e));
                    }
                },
                Event::VariableCreated(site_uid, result) => {
                    self.is_loading = false;
                    match result {
                        Ok(_) => {
                            // Refresh variables
                            self.fetch_site_variables(site_uid, events.sender());
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                Event::VariableUpdated(site_uid, result) => {
                    self.is_loading = false;
                    match result {
                        Ok(updated_var) => {
                            // Update local state in place
                            if let Some(site) = self.sites.iter_mut().find(|s| s.uid == site_uid) {
                                if let Some(vars) = &mut site.variables {
                                    if let Some(var) =
                                        vars.iter_mut().find(|v| v.id == updated_var.id)
                                    {
                                        *var = updated_var;
                                    }
                                }
                            }
                            // Note: No need to re-fetch variables, providing immediate feedback!
                        }
                        Err(e) => self.error = Some(e),
                    }
                }

                Event::SiteUpdated(result) => {
                    self.is_loading = false;
                    match result {
                        Ok(updated_site) => {
                            // Find and update the site in the local list
                            if let Some(index) =
                                self.sites.iter().position(|s| s.uid == updated_site.uid)
                            {
                                // Preserve variables as they are not returned in the update response (skipped)
                                let vars = self.sites[index].variables.clone();
                                self.sites[index] = updated_site;
                                self.sites[index].variables = vars;

                                // If this is the currently selected site, update the edit state to reflect changes in UI
                                if let Some(selected_idx) = self.table_state.selected() {
                                    if selected_idx == index {
                                        self.populate_site_edit_state();
                                    }
                                }
                            }
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
            }
        }
        Ok(())
    }

    fn fetch_rocket_incidents(&self, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if let Some(client) = &self.rocket_client {
            let client = client.clone();
            tokio::spawn(async move {
                let result = client.get_incidents().await.map_err(|e| e.to_string());
                tx.send(Event::IncidentsFetched(result)).unwrap();
            });
        }
    }

    fn fetch_sites(&mut self, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if let Some(client) = &self.client {
            self.is_loading = true;
            self.error = None;
            let client = client.clone();
            let page = self.current_page;
            tokio::spawn(async move {
                let result = client
                    .get_sites(page, 50, None)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::SitesFetched(result)).unwrap();
            });
        }
    }

    fn fetch_devices(&mut self, site_uid: String, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if let Some(client) = &self.client {
            self.devices_loading = true;
            self.devices_error = None;
            self.devices = Vec::new(); // Clear previous
            let client = client.clone();
            tokio::spawn(async move {
                // Fetch first page of devices for now
                let result = client
                    .get_devices(&site_uid, 0, 50)
                    .await
                    .map_err(|e| format!("{:#}", e));
                tx.send(Event::DevicesFetched(result)).unwrap();
            });
        }
    }

    fn fetch_site_variables(
        &self,
        site_uid: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.client {
            let client = client.clone();
            tokio::spawn(async move {
                let result = client
                    .get_site_variables(&site_uid)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::SiteVariablesFetched(site_uid, result))
                    .unwrap();
            });
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        // DEBUG LOG
        /*
        let _ = std::fs::OpenOptions::new().create(true).append(true).open("debug.log").map(|mut f| {
             use std::io::Write;
             writeln!(f, "Key Event: {:?} | Mode: {:?}", key.code, self.input_state.mode).unwrap();
        });
        */
        // Handle Input Mode first
        if self.input_state.mode == InputMode::Editing {
            match key.code {
                KeyCode::Esc => {
                    self.input_state.mode = InputMode::Normal;
                }
                KeyCode::Enter => {
                    // Check if we are editing a setting or a variable
                    if let Some(field) = self.input_state.editing_setting {
                        // Update the corresponding field in site_edit_state from the buffer
                        match field {
                            SiteEditField::Name => {
                                self.site_edit_state.name = self.input_state.name_buffer.clone()
                            }
                            SiteEditField::Description => {
                                self.site_edit_state.description =
                                    self.input_state.name_buffer.clone()
                            }
                            SiteEditField::Notes => {
                                self.site_edit_state.notes = self.input_state.name_buffer.clone()
                            }
                        }
                        self.submit_site_update(tx);
                    } else {
                        // Variable Submit
                        self.submit_variable(tx);
                    }
                    self.input_state.mode = InputMode::Normal;
                }
                KeyCode::Tab => {
                    // Switch field
                    self.input_state.active_field = match self.input_state.active_field {
                        InputField::Name => InputField::Value,
                        InputField::Value => InputField::Name,
                        // No tab switching for simple single-field settings edits for now, keep it simple
                        _ => self.input_state.active_field,
                    };
                }
                KeyCode::Backspace => {
                    match self.input_state.active_field {
                        InputField::Name
                        | InputField::SiteName
                        | InputField::SiteDescription
                        | InputField::SiteNotes => {
                            self.input_state.name_buffer.pop();
                        }
                        InputField::Value => {
                            self.input_state.value_buffer.pop();
                        }
                    };
                }
                KeyCode::Char(c) => {
                    match self.input_state.active_field {
                        InputField::Name
                        | InputField::SiteName
                        | InputField::SiteDescription
                        | InputField::SiteNotes => {
                            self.input_state.name_buffer.push(c);
                        }
                        InputField::Value => {
                            self.input_state.value_buffer.push(c);
                        }
                    };
                }
                _ => {}
            }
            return;
        }

        match self.current_view {
            CurrentView::List => match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
                KeyCode::Char('n') | KeyCode::Right => self.next_page(tx),
                KeyCode::Char('p') | KeyCode::Left => self.prev_page(tx),
                KeyCode::Char('r') => {
                    self.fetch_sites(tx);
                }
                KeyCode::Enter => {
                    if let Some(idx) = self.table_state.selected() {
                        if let Some(site) = self.sites.get(idx) {
                            self.current_view = CurrentView::Detail;
                            self.fetch_devices(site.uid.clone(), tx);
                        }
                    }
                }
                _ => {}
            },
            CurrentView::Detail => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.current_view = CurrentView::List;
                }
                KeyCode::Tab => {
                    self.detail_tab = match self.detail_tab {
                        SiteDetailTab::Devices => SiteDetailTab::Variables,
                        SiteDetailTab::Variables => SiteDetailTab::Settings,
                        SiteDetailTab::Settings => SiteDetailTab::Devices,
                    };

                    // Populate Settings state when switching to it
                    if self.detail_tab == SiteDetailTab::Settings {
                        self.populate_site_edit_state();
                    }
                }
                // Determine context based on tab
                KeyCode::Char('j') | KeyCode::Down => match self.detail_tab {
                    SiteDetailTab::Devices => self.next_device(),
                    SiteDetailTab::Variables => self.next_variable(),
                    SiteDetailTab::Settings => self.next_setting(),
                },
                KeyCode::Char('k') | KeyCode::Up => match self.detail_tab {
                    SiteDetailTab::Devices => self.prev_device(),
                    SiteDetailTab::Variables => self.prev_variable(),
                    SiteDetailTab::Settings => self.prev_setting(),
                },
                KeyCode::Char('e') => {
                    if self.detail_tab == SiteDetailTab::Variables {
                        self.open_edit_variable_modal();
                    } else if self.detail_tab == SiteDetailTab::Settings {
                        self.open_edit_setting_modal();
                    }
                }
                // Variable Actions (Enter/Space on "Create +" row)
                KeyCode::Enter | KeyCode::Char(' ')
                    if self.detail_tab == SiteDetailTab::Variables =>
                {
                    if let Some(idx) = self.variables_table_state.selected() {
                        if let Some(site_idx) = self.table_state.selected() {
                            if let Some(site) = self.sites.get(site_idx) {
                                let var_count =
                                    site.variables.as_ref().map(|v| v.len()).unwrap_or(0);
                                if idx == var_count {
                                    self.open_create_variable_modal();
                                } else {
                                    self.open_edit_variable_modal();
                                }
                            }
                        }
                    }
                }
                // Settings Actions
                KeyCode::Char(' ') | KeyCode::Enter
                    if self.detail_tab == SiteDetailTab::Settings =>
                {
                    // Toggle boolean settings for quick action, or submit if purely selecting
                    self.toggle_setting(tx.clone());
                }
                _ => {}
            },
        }
    }

    fn open_create_variable_modal(&mut self) {
        self.input_state = InputState {
            mode: InputMode::Editing,
            name_buffer: String::new(),
            value_buffer: String::new(),
            active_field: InputField::Name,
            is_creating: true,
            editing_variable_id: None,
            editing_setting: None,
        };
    }

    fn open_edit_variable_modal(&mut self) {
        if let Some(idx) = self.variables_table_state.selected() {
            if let Some(site_idx) = self.table_state.selected() {
                if let Some(site) = self.sites.get(site_idx) {
                    if let Some(vars) = &site.variables {
                        if let Some(var) = vars.get(idx) {
                            // DEBUG LOGGING
                            let _ = std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open("debug.log")
                                .map(|mut f| {
                                    use std::io::Write;
                                    writeln!(
                                        f,
                                        "Opening Edit Modal for variable: {} - Value: {}",
                                        var.name, var.value
                                    )
                                    .unwrap();
                                });
                            self.input_state = InputState {
                                mode: InputMode::Editing,
                                name_buffer: var.name.clone(),
                                value_buffer: var.value.clone(), // Note: Masked values might be empty/hidden
                                active_field: InputField::Value, // Start on Value usually for edits
                                is_creating: false,
                                editing_variable_id: Some(var.id),
                                editing_setting: None,
                            };
                        }
                    }
                }
            }
        }
    }

    fn submit_variable(&mut self, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if let Some(idx) = self.table_state.selected() {
            if let Some(site) = self.sites.get(idx).cloned() {
                let site_uid = site.uid;
                let client = self.client.as_ref().unwrap().clone();
                let name = self.input_state.name_buffer.clone();
                let value = self.input_state.value_buffer.clone();

                if self.input_state.is_creating {
                    // Create
                    tokio::spawn(async move {
                        let req = CreateVariableRequest {
                            name,
                            value,
                            masked: false, // Default to false for now
                        };
                        let result = client
                            .create_site_variable(&site_uid, req)
                            .await
                            .map_err(|e| e.to_string());
                        tx.send(Event::VariableCreated(site_uid, result)).unwrap();
                    });
                } else if let Some(id) = self.input_state.editing_variable_id {
                    // Update
                    tokio::spawn(async move {
                        let req = UpdateVariableRequest { name, value };
                        let result = client
                            .update_site_variable(&site_uid, id, req)
                            .await
                            .map_err(|e| e.to_string());
                        tx.send(Event::VariableUpdated(site_uid, result)).unwrap();
                    });
                }
            }
        }
    }

    fn populate_site_edit_state(&mut self) {
        if let Some(idx) = self.table_state.selected() {
            if let Some(site) = self.sites.get(idx) {
                // DEBUG LOGGING
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("debug.log")
                    .map(|mut f| {
                        use std::io::Write;
                        writeln!(
                            f,
                            "Populating state from site: {} - Desc: {:?}",
                            site.name, site.description
                        )
                        .unwrap();
                    });

                self.site_edit_state = SiteEditState {
                    name: site.name.clone(),
                    description: site.description.clone().unwrap_or_default(),
                    notes: site.notes.clone().unwrap_or_default(),
                    on_demand: site.on_demand.unwrap_or(false),
                    splashtop_auto_install: site.splashtop_auto_install.unwrap_or(false),
                    active_field: SiteEditField::Name,
                    is_editing: true,
                };
            }
        }
    }

    fn submit_site_update(&mut self, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if let Some(idx) = self.table_state.selected() {
            if let Some(site) = self.sites.get(idx).cloned() {
                let site_uid = site.uid;
                let client = self.client.as_ref().unwrap().clone();
                let req = UpdateSiteRequest {
                    name: self.site_edit_state.name.clone(),
                    description: Some(self.site_edit_state.description.clone()),
                    notes: Some(self.site_edit_state.notes.clone()),
                    on_demand: Some(self.site_edit_state.on_demand),
                    splashtop_auto_install: Some(self.site_edit_state.splashtop_auto_install),
                };

                // DEBUG LOG
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("debug.log")
                    .map(|mut f| {
                        use std::io::Write;
                        writeln!(f, "Submitting Site Update for UID: {}", site_uid).unwrap();
                        writeln!(f, "Payload: {:?}", req).unwrap();
                    });

                tokio::spawn(async move {
                    let result = client
                        .update_site(&site_uid, req)
                        .await
                        .map_err(|e| e.to_string());
                    tx.send(Event::SiteUpdated(result)).unwrap();
                });
            }
        }
    }

    fn next_variable(&mut self) {
        if let Some(site_idx) = self.table_state.selected() {
            if let Some(site) = self.sites.get(site_idx) {
                // Allow selecting up to len() (which is the "Create +" button)
                let count = site.variables.as_ref().map(|v| v.len()).unwrap_or(0);

                let i = match self.variables_table_state.selected() {
                    Some(i) => {
                        if i >= count {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.variables_table_state.select(Some(i));
            }
        }
    }

    fn prev_variable(&mut self) {
        if let Some(site_idx) = self.table_state.selected() {
            if let Some(site) = self.sites.get(site_idx) {
                let count = site.variables.as_ref().map(|v| v.len()).unwrap_or(0);

                let i = match self.variables_table_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            count
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.variables_table_state.select(Some(i));
            }
        }
    }

    fn next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.sites.len().saturating_sub(1) {
                    0 // Loop back to top
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.sites.len().saturating_sub(1) // Loop to bottom
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn next_page(&mut self, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if self.current_page + 1 < self.total_pages {
            self.current_page += 1;
            self.fetch_sites(tx);
        }
    }

    fn prev_page(&mut self, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.fetch_sites(tx);
        }
    }

    fn next_device(&mut self) {
        let i = match self.devices_table_state.selected() {
            Some(i) => {
                if i >= self.devices.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.devices_table_state.select(Some(i));
    }

    fn prev_device(&mut self) {
        let i = match self.devices_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.devices.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.devices_table_state.select(Some(i));
    }
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
}
