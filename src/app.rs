use crate::api::datto::DattoClient;
use crate::app_helpers::generate_job_rows;
use crate::api::datto::activity::ActivityApi;
use crate::api::datto::devices::DevicesApi;
use crate::api::datto::jobs::JobsApi;
use crate::api::datto::sites::SitesApi;
use crate::api::datto::types::{
    ActivityLog, Component, CreateVariableRequest, Device, JobResult, QuickJobComponent,
    QuickJobRequest, QuickJobResponse, QuickJobVariable, Site, UpdateSiteRequest,
    UpdateVariableRequest,
};
use crate::api::datto::variables::VariablesApi;
use crate::event::{Event, EventHandler};
use crate::tui::Tui;
use crate::ui;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;

use crate::api::datto_av::DattoAvClient;
use crate::api::datto_av::types::AgentDetail;
use crate::api::rocket_cyber::RocketCyberClient;
use crate::api::rocket_cyber::incidents::IncidentsApi;
use crate::api::sophos::{Endpoint, SophosClient};
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
    DeviceDetail,
    ActivityDetail,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SiteDetailTab {
    Devices,
    Variables,
    Settings,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DeviceDetailTab {
    OpenAlerts,
    Activities,
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

#[derive(Debug, PartialEq, Clone)]
pub enum JobViewRow {
    ComponentHeader(usize), // Component Index
    StdOutLink(usize),      // Component Index
    StdErrLink(usize),      // Component Index
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RunComponentStep {
    Search,
    FillVariables,
    Review,
    Result,
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
    pub rocket_client: Option<RocketCyberClient>,
    pub sophos_client: Option<SophosClient>,
    pub datto_av_client: Option<DattoAvClient>,
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
    pub selected_device: Option<Device>,
    pub device_detail_tab: DeviceDetailTab,

    // Activity Logs
    pub activity_logs: Vec<ActivityLog>,
    pub activity_logs_loading: bool,
    pub activity_logs_error: Option<String>,
    pub activity_logs_table_state: TableState,

    // Open Alerts
    pub open_alerts: Vec<crate::api::datto::types::Alert>,
    pub open_alerts_loading: bool,
    pub open_alerts_error: Option<String>,
    pub open_alerts_table_state: TableState,

    // Job Results
    pub selected_activity_log: Option<ActivityLog>,
    pub selected_job_result: Option<JobResult>,
    pub job_result_loading: bool,
    pub job_result_error: Option<String>,
    pub selected_job_row_index: usize,

    // Site & Device Editing State
    pub variables_table_state: TableState,
    pub udf_table_state: TableState,
    pub editing_udf_index: Option<usize>,
    pub site_edit_state: SiteEditState,
    pub settings_table_state: TableState,
    pub input_state: InputState,

    pub sophos_endpoints: HashMap<String, Endpoint>,
    pub sophos_loading: HashMap<String, bool>,

    pub datto_av_agents: HashMap<String, AgentDetail>,
    pub datto_av_loading: HashMap<String, bool>,
    // Store alerts/policies per hostname
    pub datto_av_alerts: HashMap<String, Vec<crate::api::datto_av::types::Alert>>,
    pub datto_av_policies: HashMap<String, serde_json::Value>,

    pub scan_status: HashMap<String, crate::event::ScanStatus>,

    // Job Output Popup
    pub show_popup: bool,
    pub popup_title: String,
    pub popup_content: String,
    pub popup_loading: bool,

    // Device Search Popup
    pub show_device_search: bool,
    pub device_search_query: String,
    pub device_search_results: Vec<Device>,
    pub device_search_loading: bool,
    pub device_search_error: Option<String>,
    pub device_search_table_state: TableState,
    pub last_search_input: Option<std::time::Instant>,
    pub last_searched_query: String,

    // Device Variables Popup
    pub show_device_variables: bool,

    // Run Component Popup
    pub show_run_component: bool,
    pub run_component_step: RunComponentStep,
    pub components: Vec<Component>,
    pub filtered_components: Vec<Component>,
    pub component_search_query: String,
    pub component_list_state: TableState,
    pub selected_component: Option<Component>,
    pub component_variables: Vec<QuickJobVariable>,
    pub component_variable_index: usize,
    pub component_variable_input: String,
    pub last_job_response: Option<QuickJobResponse>,
    pub component_error: Option<String>,
    pub components_loading: bool,
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
            rocket_client: None,
            sophos_client: None,
            datto_av_client: None,
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
            selected_device: None,
            device_detail_tab: DeviceDetailTab::OpenAlerts,
            // Removed duplicates
            // variables_table_state: TableState::default(),
            // udf_table_state: TableState::default(),
            // editing_udf_index: None,

            activity_logs: Vec::new(),
            activity_logs_loading: false,
            activity_logs_error: None,
            activity_logs_table_state: TableState::default(),

            open_alerts: Vec::new(),
            open_alerts_loading: false,
            open_alerts_error: None,
            open_alerts_table_state: TableState::default(),

            selected_activity_log: None,
            selected_job_result: None,
            job_result_loading: false,
            job_result_error: None,
            selected_job_row_index: 0,

            variables_table_state: TableState::default(),
            udf_table_state: TableState::default(),
            editing_udf_index: None,
            site_edit_state: SiteEditState::default(),
            settings_table_state: TableState::default(),
            input_state: InputState::default(),

            sophos_endpoints: HashMap::new(),
            sophos_loading: HashMap::new(),

            datto_av_agents: HashMap::new(),
            datto_av_loading: HashMap::new(),
            datto_av_alerts: HashMap::new(),
            datto_av_policies: HashMap::new(),

            scan_status: HashMap::new(),

            show_popup: false,
            popup_title: String::new(),
            popup_content: String::new(),
            popup_loading: false,

            // Device Search Popup
            show_device_search: false,
            device_search_query: String::new(),
            device_search_results: Vec::new(),
            device_search_loading: false,
            device_search_error: None,
            device_search_table_state: TableState::default(),
            last_search_input: None,
            last_searched_query: String::new(),

            show_device_variables: false,

            show_run_component: false,
            run_component_step: RunComponentStep::Search,
            components: Vec::new(),
            filtered_components: Vec::new(),
            component_search_query: String::new(),
            component_list_state: TableState::default(),
            selected_component: None,
            component_variables: Vec::new(),
            component_variable_index: 0,
            component_variable_input: String::new(),
            last_job_response: None,
            component_error: None,
            components_loading: false,
        }
    }
}

impl App {
    pub fn new(
        client: Option<DattoClient>,
        rocket_client: Option<RocketCyberClient>,
        sophos_client: Option<SophosClient>,
        datto_av_client: Option<DattoAvClient>,
    ) -> Self {
        let mut app = Self::default();
        app.client = client;
        app.rocket_client = rocket_client;
        app.sophos_client = sophos_client;
        app.datto_av_client = datto_av_client;
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

        // Authenticate Sophos if present
        if let Some(client) = &mut self.sophos_client {
            if let Err(e) = client.authenticate().await {
                self.error = Some(format!("Sophos Auth Failed: {}", e));
            }
        }

        while !self.should_quit {
            tui.draw(|f| {
                ui::render(self, f);
            })?;

            match events.next().await? {
                Event::Key(key) => self.handle_key_event(key, events.sender()),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                event => self.handle_event(event, events.sender()).await?,
            }
        }
        Ok(())
    }

    async fn handle_event(
        &mut self,
        event: Event,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) -> Result<()> {
        match event {
            Event::Tick => {
                // Handle Device Search Debounce
                if self.show_device_search {
                    if let Some(last_input) = self.last_search_input {
                        if last_input.elapsed() >= std::time::Duration::from_millis(500) {
                             // Log debounce check
                             let _ = std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open("debug.log")
                                .map(|mut f| {
                                     use std::io::Write;
                                     writeln!(f, "Tick: Checking search. Query='{}', Last='{}'", self.device_search_query, self.last_searched_query).unwrap();
                                });

                            if self.device_search_query.len() >= 3
                                && self.device_search_query != self.last_searched_query
                            {
                                self.last_searched_query = self.device_search_query.clone();
                                self.search_devices(self.device_search_query.clone(), tx.clone());
                            }
                        }
                    }
                }
            }
            Event::Key(_) | Event::Mouse(_) | Event::Resize(_, _) => {}
            Event::DeviceSearchResultsFetched(result) => {
                self.device_search_loading = false;
                match result {
                    Ok(response) => {
                        self.device_search_results = response.devices;
                        if !self.device_search_results.is_empty() {
                            self.device_search_table_state.select(Some(0));
                        } else {
                            self.device_search_table_state.select(None);
                        }
                    }
                    Err(e) => {
                        self.device_search_error = Some(e);
                    }
                }
            }
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
                                self.fetch_site_variables(site.uid.clone(), tx.clone());
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
                        site.variables = Some(variables.clone());

                        // Check for Sophos MDR
                        for var in &variables {
                            if var.name == "tuiMdrProvider" && var.value == "Sophos" {
                                // Find tuiMdrId
                                if let Some(id_var) =
                                    variables.iter().find(|v| v.name == "tuiMdrId")
                                {
                                    // Check for tuiMdrRegion to skip tenant call
                                    let region = variables
                                        .iter()
                                        .find(|v| v.name == "tuiMdrRegion")
                                        .map(|v| v.value.clone());

                                    self.fetch_sophos_cases(
                                        id_var.value.clone(),
                                        region,
                                        tx.clone(),
                                    );
                                }
                            }
                        }
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
                        self.fetch_site_variables(site_uid, tx.clone());
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
                                if let Some(var) = vars.iter_mut().find(|v| v.id == updated_var.id)
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
            Event::SophosCasesFetched(tenant_id, result) => match result {
                Ok(cases) => {
                    // Update stats
                    let entry = self
                        .incident_stats
                        .entry(tenant_id.clone())
                        .or_insert(IncidentStats::default());

                    // Reset or accumulate? Probably reset for this tenant as it's a fresh fetch
                    entry.active = 0;
                    entry.resolved = 0;

                    for case in cases {
                        let status = case.status.as_deref().unwrap_or("").to_lowercase();
                        if status == "resolved" || status == "closed" {
                            // Assuming closed is also resolved
                            entry.resolved += 1;
                        } else {
                            entry.active += 1;
                        }
                    }
                }
                Err(e) => {
                    self.error = Some(format!(
                        "Failed to fetch Sophos cases for {}: {}",
                        tenant_id, e
                    ));
                }
            },
            Event::SophosEndpointsFetched(hostname, result) => {
                self.sophos_loading.insert(hostname.clone(), false);
                match result {
                    Ok(endpoints) => {
                        if let Some(endpoint) = endpoints.first() {
                            self.sophos_endpoints
                                .insert(hostname.clone(), endpoint.clone());

                            // Cache Endpoint ID in UDF 30 if different
                            if let Some(device) =
                                self.devices.iter().find(|d| d.hostname == hostname)
                            {
                                let current_udf30 = device
                                    .udf
                                    .as_ref()
                                    .and_then(|u| u.udf30.as_ref())
                                    .map(|s| s.as_str())
                                    .unwrap_or("");
                                if current_udf30 != endpoint.id {
                                    // Update UDF 30 using DevicesApi
                                    if let Some(client) = &self.client {
                                        let device_uid = device.uid.clone();
                                        let endpoint_id = endpoint.id.clone();
                                        let client = client.clone();
                                        tokio::spawn(async move {
                                            let udf = crate::api::datto::types::Udf {
                                                udf30: Some(endpoint_id),
                                                udf1: None,
                                                udf2: None,
                                                udf3: None,
                                                udf4: None,
                                                udf5: None,
                                                udf6: None,
                                                udf7: None,
                                                udf8: None,
                                                udf9: None,
                                                udf10: None,
                                                udf11: None,
                                                udf12: None,
                                                udf13: None,
                                                udf14: None,
                                                udf15: None,
                                                udf16: None,
                                                udf17: None,
                                                udf18: None,
                                                udf19: None,
                                                udf20: None,
                                                udf21: None,
                                                udf22: None,
                                                udf23: None,
                                                udf24: None,
                                                udf25: None,
                                                udf26: None,
                                                udf27: None,
                                                udf28: None,
                                                udf29: None,
                                            };

                                            let _ =
                                                client.update_device_udf(&device_uid, &udf).await;
                                        });
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!(
                            "Failed to fetch Sophos endpoint for {}: {}",
                            hostname, e
                        ));
                    }
                }
            }
            Event::SophosScanStarted(hostname, result) => {
                match result {
                    Ok(_) => {
                        // Scan started logic: wait 2 seconds then update status
                        let h = hostname.clone();
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            tx_clone
                                .send(Event::ScanStatusChanged(
                                    h,
                                    crate::event::ScanStatus::Started,
                                ))
                                .unwrap();
                        });
                    }
                    Err(e) => {
                        self.scan_status.remove(&hostname);
                        self.error = Some(format!("Failed to start scan for {}: {}", hostname, e));
                    }
                }
            }
            Event::DattoAvAgentFetched(hostname, result) => {
                self.datto_av_loading.insert(hostname.clone(), false);
                match result {
                    Ok(agent) => {
                        self.datto_av_agents.insert(hostname.clone(), agent.clone());

                        // Check/Update UDF 30 if needed
                        // We only update if we found it via hostname (implying we might not have had ID)
                        // OR just check if UDF 30 matches.
                        // Check/Update UDF 30 if needed
                        // First, find the index of the device to update to avoid borrow issues
                        if let Some(dev_idx) =
                            self.devices.iter().position(|d| d.hostname == hostname)
                        {
                            let device_uid = self.devices[dev_idx].uid.clone();
                            let current_udf30 = self.devices[dev_idx]
                                .udf
                                .as_ref()
                                .and_then(|u| u.udf30.as_ref())
                                .map(|s| s.as_str())
                                .unwrap_or("")
                                .to_string();

                            if current_udf30 != agent.id {
                                // Update UDF 30
                                // Update local state immediately for responsiveness
                                if let Some(udfs) = &mut self.devices[dev_idx].udf {
                                    udfs.udf30 = Some(agent.id.clone());
                                } else {
                                    let mut new_udf = crate::api::datto::types::Udf::default();
                                    new_udf.udf30 = Some(agent.id.clone());
                                    self.devices[dev_idx].udf = Some(new_udf);
                                }

                                // Also update selected device if it matches
                                if let Some(sel) = &mut self.selected_device {
                                    if sel.uid == device_uid {
                                        if let Some(udfs) = &mut sel.udf {
                                            udfs.udf30 = Some(agent.id.clone());
                                        } else {
                                            let mut new_udf =
                                                crate::api::datto::types::Udf::default();
                                            new_udf.udf30 = Some(agent.id.clone());
                                            sel.udf = Some(new_udf);
                                        }
                                    }
                                }

                                if let Some(client) = &self.client {
                                    let agent_id = agent.id.clone();
                                    let client = client.clone();
                                    tokio::spawn(async move {
                                        let udf = crate::api::datto::types::Udf {
                                            udf30: Some(agent_id),
                                            udf1: None,
                                            udf2: None,
                                            udf3: None,
                                            udf4: None,
                                            udf5: None,
                                            udf6: None,
                                            udf7: None,
                                            udf8: None,
                                            udf9: None,
                                            udf10: None,
                                            udf11: None,
                                            udf12: None,
                                            udf13: None,
                                            udf14: None,
                                            udf15: None,
                                            udf16: None,
                                            udf17: None,
                                            udf18: None,
                                            udf19: None,
                                            udf20: None,
                                            udf21: None,
                                            udf22: None,
                                            udf23: None,
                                            udf24: None,
                                            udf25: None,
                                            udf26: None,
                                            udf27: None,
                                            udf28: None,
                                            udf29: None,
                                        };
                                        let _ = client.update_device_udf(&device_uid, &udf).await;
                                    });
                                }
                            }
                        }

                        // Fetch alerts for this agent
                        self.fetch_datto_av_alerts(agent.id.clone(), hostname.clone(), tx.clone());
                        // Fetch policies for this agent
                        self.fetch_datto_av_policies(agent.id.clone(), hostname, tx.clone());
                    }
                    Err(e) => {
                        self.error = Some(format!(
                            "Failed to fetch Datto AV agent for {}: {}",
                            hostname, e
                        ));
                    }
                }
            }
            Event::DattoAvScanStarted(hostname, result) => {
                match result {
                    Ok(_) => {
                        // Scan started logic: wait 2 seconds then update status
                        let h = hostname.clone();
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            tx_clone
                                .send(Event::ScanStatusChanged(
                                    h,
                                    crate::event::ScanStatus::Started,
                                ))
                                .unwrap();
                        });
                    }
                    Err(e) => {
                        self.scan_status.remove(&hostname);
                        self.error = Some(format!(
                            "Failed to start Datto AV scan for {}: {}",
                            hostname, e
                        ));
                    }
                }
            }
            Event::ScanStatusChanged(hostname, status) => {
                self.scan_status.insert(hostname, status);
            }
            Event::DattoAvAlertsFetched(hostname, result) => match result {
                Ok(alerts) => {
                    self.datto_av_alerts.insert(hostname, alerts);
                }
                Err(_e) => {
                    // Ignore error for now, or log it
                }
            },
            Event::DattoAvPoliciesFetched(hostname, result) => match result {
                Ok(policies) => {
                    // Log to debug.log
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("debug.log")
                        .map(|mut f| {
                            use std::io::Write;
                            writeln!(f, "Policies for {}: {:#?}", hostname, policies).unwrap();
                        });
                    self.datto_av_policies.insert(hostname, policies);
                }
                Err(e) => {
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("debug.log")
                        .map(|mut f| {
                            use std::io::Write;
                            writeln!(f, "Failed to fetch policies for {}: {}", hostname, e).unwrap();
                        });
                }
            },
            Event::ActivityLogsFetched(result) => {
                self.activity_logs_loading = false;
                match result {
                    Ok(response) => {
                        self.activity_logs = response.activities;
                        if !self.activity_logs.is_empty() {
                            self.activity_logs_table_state.select(Some(0));
                        } else {
                            self.activity_logs_table_state.select(None);
                        }
                    }
                    Err(e) => {
                        self.activity_logs_error = Some(e);
                    }
                }
            }
            Event::OpenAlertsFetched(device_uid, result) => {
                // Ensure the result corresponds to the currently selected device
                if let Some(device) = &self.selected_device {
                    if device.uid == device_uid {
                        self.open_alerts_loading = false;
                        match result {
                            Ok(alerts) => {
                                // Debug log
                                let _ = std::fs::OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open("debug.log")
                                    .map(|mut f| {
                                        use std::io::Write;
                                        writeln!(f, "Fetched {} alerts for device {}", alerts.len(), device_uid).unwrap();
                                        writeln!(f, "Alerts Data: {:#?}", alerts).unwrap();
                                    });

                                self.open_alerts = alerts;
                                if !self.open_alerts.is_empty() {
                                    self.open_alerts_table_state.select(Some(0));
                                } else {
                                    self.open_alerts_table_state.select(None);
                                }
                            }
                            Err(e) => {
                                // Debug log error
                                let _ = std::fs::OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open("debug.log")
                                    .map(|mut f| {
                                        use std::io::Write;
                                        writeln!(f, "Error fetching alerts for {}: {}", device_uid, e).unwrap();
                                    });
                                self.open_alerts_error = Some(e);
                            }
                        }
                    }
                }
            }
            Event::JobResultFetched(result) => {
                self.job_result_loading = false;
                match result {
                    Ok(job_result) => {
                        self.selected_job_result = Some(job_result);
                    }
                    Err(e) => {
                        self.job_result_error = Some(e);
                    }
                }
            }
            Event::JobStdOutFetched(result) => {
                self.popup_loading = false;
                match result {
                    Ok(outputs) => {
                        // Find the output for the selected component (derived from selected row)
                        if let Some(job_result) = &self.selected_job_result {
                            let rows = generate_job_rows(job_result);
                            if let Some(row) = rows.get(self.selected_job_row_index) {
                                let comp_idx = match row {
                                    JobViewRow::ComponentHeader(i)
                                    | JobViewRow::StdOutLink(i)
                                    | JobViewRow::StdErrLink(i) => *i,
                                };

                                if let Some(components) = &job_result.component_results {
                                    if let Some(selected_comp) = components.get(comp_idx) {
                                        if let Some(comp_uid) = &selected_comp.component_uid {
                                            if let Some(output) = outputs
                                                .iter()
                                                .find(|o| o.component_uid.as_ref() == Some(comp_uid))
                                            {
                                                self.popup_content = output
                                                    .std_data
                                                    .clone()
                                                    .unwrap_or_else(|| "No StdOut data".to_string());
                                            } else {
                                                self.popup_content =
                                                    "No StdOut found for this component".to_string();
                                            }
                                        } else {
                                            self.popup_content = "Component UID missing".to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.popup_content = format!("Error: {}", e);
                    }
                }
            }
            Event::JobStdErrFetched(result) => {
                self.popup_loading = false;
                match result {
                    Ok(outputs) => {
                        if let Some(job_result) = &self.selected_job_result {
                            let rows = generate_job_rows(job_result);
                            if let Some(row) = rows.get(self.selected_job_row_index) {
                                let comp_idx = match row {
                                    JobViewRow::ComponentHeader(i)
                                    | JobViewRow::StdOutLink(i)
                                    | JobViewRow::StdErrLink(i) => *i,
                                };

                                if let Some(components) = &job_result.component_results {
                                    if let Some(selected_comp) = components.get(comp_idx) {
                                        if let Some(comp_uid) = &selected_comp.component_uid {
                                            if let Some(output) = outputs
                                                .iter()
                                                .find(|o| o.component_uid.as_ref() == Some(comp_uid))
                                            {
                                                self.popup_content = output
                                                    .std_data
                                                    .clone()
                                                    .unwrap_or_else(|| "No StdErr data".to_string());
                                            } else {
                                                self.popup_content =
                                                    "No StdErr found for this component".to_string();
                                            }
                                        } else {
                                            self.popup_content = "Component UID missing".to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.popup_content = format!("Error: {}", e);
                    }
                }
            }
            Event::ComponentsFetched(result) => {
                self.components_loading = false;
                match result {
                    Ok(response) => {
                        self.components = response.components;
                        // Sort by name
                        self.components
                            .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                        
                        // Initial filter (all)
                        self.filtered_components = self.components.clone();
                        
                        if !self.filtered_components.is_empty() {
                            self.component_list_state.select(Some(0));
                        } else {
                            self.component_list_state.select(None);
                        }
                    }
                    Err(e) => {
                        self.component_error = Some(e);
                    }
                }
            }
            Event::QuickJobExecuted(result) => {
                self.components_loading = false;
                match result {
                    Ok(response) => {
                        self.last_job_response = Some(response);
                        self.run_component_step = RunComponentStep::Result;
                    }
                    Err(e) => {
                        self.component_error = Some(e);
                        self.run_component_step = RunComponentStep::Result;
                    }
                }
            }
        }

        Ok(())
    }

    fn fetch_components(&mut self, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if let Some(client) = &self.client {
            self.components_loading = true;
            self.component_error = None;
            self.components.clear();
            self.filtered_components.clear();
            
            let client = client.clone();
            tokio::spawn(async move {
                // Fetch first page, maybe loop if needed but start with one page
                let result = client.get_components(None).await.map_err(|e| e.to_string());
                tx.send(Event::ComponentsFetched(result)).unwrap();
            });
        }
    }

    fn run_component_job(&mut self, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if let Some(client) = &self.client {
            if let Some(device) = &self.selected_device {
                if let Some(component) = &self.selected_component {
                    self.components_loading = true;
                    self.component_error = None;
                    
                    let client = client.clone();
                    let device_uid = device.uid.clone();
                    let req = QuickJobRequest {
                        job_name: format!("Run Component: {}", component.name),
                        job_component: QuickJobComponent {
                            component_uid: component.uid.clone(),
                            variables: self.component_variables.clone(),
                        },
                    };

                    tokio::spawn(async move {
                        let result = client.run_quick_job(&device_uid, req).await.map_err(|e| format!("{:#}", e));
                        tx.send(Event::QuickJobExecuted(result)).unwrap();
                    });
                }
            }
        }
    }

    fn filter_components(&mut self) {
        if self.component_search_query.is_empty() {
            self.filtered_components = self.components.clone();
        } else {
            let query = self.component_search_query.to_lowercase();
            self.filtered_components = self.components
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&query))
                .cloned()
                .collect();
        }
        
        // Reset selection
        if !self.filtered_components.is_empty() {
            self.component_list_state.select(Some(0));
        } else {
            self.component_list_state.select(None);
        }
    }

    fn handle_run_component_input(&mut self, key: KeyEvent, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        match self.run_component_step {
            RunComponentStep::Search => {
                match key.code {
                    KeyCode::Esc => {
                        self.show_run_component = false;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(i) = self.component_list_state.selected() {
                            let next = if i >= self.filtered_components.len().saturating_sub(1) {
                                0
                            } else {
                                i + 1
                            };
                            self.component_list_state.select(Some(next));
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(i) = self.component_list_state.selected() {
                            let next = if i == 0 {
                                self.filtered_components.len().saturating_sub(1)
                            } else {
                                i - 1
                            };
                            self.component_list_state.select(Some(next));
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(i) = self.component_list_state.selected() {
                            if let Some(comp) = self.filtered_components.get(i) {
                                self.selected_component = Some(comp.clone());
                                // Prepare variables
                                self.component_variables.clear();
                                
                                if let Some(vars) = &comp.variables {
                                    // Sort by variablesIdx if possible
                                    let mut sorted_vars = vars.clone();
                                    sorted_vars.sort_by_key(|v| v.variables_idx.unwrap_or(0));
                                    
                                    for var in sorted_vars {
                                        self.component_variables.push(QuickJobVariable {
                                            name: var.name.clone(),
                                            value: var.default_val.clone().unwrap_or_default(),
                                        });
                                    }
                                }

                                if self.component_variables.is_empty() {
                                    self.run_component_step = RunComponentStep::Review;
                                } else {
                                    self.run_component_step = RunComponentStep::FillVariables;
                                    self.component_variable_index = 0;
                                    // Initialize input buffer with first variable's default
                                    self.component_variable_input = self.component_variables[0].value.clone();
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        self.component_search_query.push(c);
                        self.filter_components();
                    }
                    KeyCode::Backspace => {
                        self.component_search_query.pop();
                        self.filter_components();
                    }
                    _ => {}
                }
            }
            RunComponentStep::FillVariables => {
                match key.code {
                    KeyCode::Esc => {
                        self.run_component_step = RunComponentStep::Search;
                    }
                    KeyCode::Enter => {
                        // Save current input to variable
                        if let Some(var) = self.component_variables.get_mut(self.component_variable_index) {
                            var.value = self.component_variable_input.clone();
                        }

                        // Move to next variable or Review
                        if self.component_variable_index < self.component_variables.len() - 1 {
                            self.component_variable_index += 1;
                            // Load next variable value into buffer
                            self.component_variable_input = self.component_variables[self.component_variable_index].value.clone();
                        } else {
                            self.run_component_step = RunComponentStep::Review;
                        }
                    }
                    KeyCode::Up => {
                        // Go back to previous variable
                        if self.component_variable_index > 0 {
                            // Save current (optional, but good UX)
                            if let Some(var) = self.component_variables.get_mut(self.component_variable_index) {
                                var.value = self.component_variable_input.clone();
                            }
                            
                            self.component_variable_index -= 1;
                            self.component_variable_input = self.component_variables[self.component_variable_index].value.clone();
                        }
                    }
                    KeyCode::Char(c) => {
                        self.component_variable_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.component_variable_input.pop();
                    }
                    _ => {}
                }
            }
            RunComponentStep::Review => {
                match key.code {
                    KeyCode::Esc => {
                        if self.component_variables.is_empty() {
                            self.run_component_step = RunComponentStep::Search;
                        } else {
                            self.run_component_step = RunComponentStep::FillVariables;
                            // Go to last variable
                            self.component_variable_index = self.component_variables.len() - 1;
                            self.component_variable_input = self.component_variables[self.component_variable_index].value.clone();
                        }
                    }
                    KeyCode::Enter => {
                        // Execute
                        self.run_component_job(tx);
                    }
                    _ => {}
                }
            }
            RunComponentStep::Result => {
                match key.code {
                    KeyCode::Enter | KeyCode::Esc => {
                        self.show_run_component = false;
                        self.run_component_step = RunComponentStep::Search;
                    }
                    _ => {}
                }
            }
        }
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

    fn search_devices(&mut self, query: String, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if let Some(client) = &self.client {
            self.device_search_loading = true;
            self.device_search_error = None;
            self.device_search_results.clear();
            
            // Log search trigger
             let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("debug.log")
                .map(|mut f| {
                     use std::io::Write;
                     writeln!(f, "Triggering API Search for: {}", query).unwrap();
                });

            let client = client.clone();
            tokio::spawn(async move {
                let result = client
                    .search_devices(&query)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::DeviceSearchResultsFetched(result)).unwrap();
            });
        }
    }

    fn fetch_activity_logs(
        &mut self,
        _device_uid: String,
        device_id: i32,
        site_id: i32,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.client {
            self.activity_logs_loading = true;
            self.activity_logs_error = None;
            self.activity_logs.clear();

            let client = client.clone();
            tokio::spawn(async move {
                // Calculate date range: last 24 hours
                let now = chrono::Utc::now();
                let yesterday = now - chrono::Duration::days(1);
                let from_str = yesterday.format("%Y-%m-%dT%H:%M:%SZ").to_string();
                let until_str = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

                // Since we cannot filter by device UID directly in the API for this endpoint (based on error message),
                // we filter by site_id and "device" entity type, then filter in memory for the specific device ID.
                let result = client
                    .get_activity_logs(
                        None,                                  // Page (None = empty/first)
                        100,                                   // Size (Increase to likely catch the device activity)
                        Some("desc".to_string()),              // Order
                        Some(from_str),                        // From (Last 24h)
                        Some(until_str),                       // Until (Now)
                        Some(vec!["device".to_string()]),      // Entities: "device" literal
                        None,                                  // Categories
                        None,                                  // Actions
                        Some(vec![site_id]),                   // SiteIds
                        None,                                  // UserIds
                    )
                    .await
                    .map(|mut response| {
                        // Client-side filtering for the specific device
                        response.activities.retain(|log| {
                            log.device_id == Some(device_id)
                        });
                        response
                    })
                    .map_err(|e| e.to_string());

                tx.send(Event::ActivityLogsFetched(result)).unwrap();
            });
        }
    }

    pub fn fetch_open_alerts(
        &mut self,
        device_uid: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = self.client.clone() {
            self.open_alerts_loading = true;
            self.open_alerts_error = None;
            self.open_alerts.clear();
            
            tokio::spawn(async move {
                let result = client
                    .get_device_open_alerts(&device_uid)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::OpenAlertsFetched(device_uid, result))
                    .unwrap();
            });
        }
    }

    fn fetch_job_result(
        &mut self,
        job_uid: String,
        device_uid: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.client {
            self.job_result_loading = true;
            self.job_result_error = None;
            self.selected_job_result = None;
            self.selected_job_row_index = 0; // Reset index

            let client = client.clone();
            tokio::spawn(async move {
                let result = client
                    .get_job_result(&job_uid, &device_uid)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::JobResultFetched(result)).unwrap();
            });
        }
    }

    fn fetch_job_stdout(
        &mut self,
        job_uid: String,
        device_uid: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.client {
            self.popup_loading = true;
            self.show_popup = true;
            self.popup_title = "StdOut".to_string();
            self.popup_content = "Loading...".to_string();

            let client = client.clone();
            tokio::spawn(async move {
                let result = client
                    .get_job_stdout(&job_uid, &device_uid)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::JobStdOutFetched(result)).unwrap();
            });
        }
    }

    fn fetch_job_stderr(
        &mut self,
        job_uid: String,
        device_uid: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.client {
            self.popup_loading = true;
            self.show_popup = true;
            self.popup_title = "StdErr".to_string();
            self.popup_content = "Loading...".to_string();

            let client = client.clone();
            tokio::spawn(async move {
                let result = client
                    .get_job_stderr(&job_uid, &device_uid)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::JobStdErrFetched(result)).unwrap();
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

    fn fetch_sophos_cases(
        &self,
        tenant_id: String,
        data_region: Option<String>,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.sophos_client {
            let client = client.clone();
            let t_id = tenant_id.clone();
            tokio::spawn(async move {
                // First get tenant to find data region IF not provided
                let cases_result = async {
                    let region = if let Some(r) = data_region {
                        r
                    } else {
                        let tenant = client.get_tenant(&t_id).await?;
                        tenant.data_region
                    };

                    let cases = client.get_cases(&t_id, &region).await?;
                    Ok(cases)
                }
                .await
                .map_err(|e: anyhow::Error| e.to_string());

                tx.send(Event::SophosCasesFetched(tenant_id, cases_result))
                    .unwrap();
            });
        }
    }

    fn fetch_sophos_endpoint(
        &mut self,
        tenant_id: String,
        data_region: Option<String>,
        hostname: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if self.sophos_endpoints.contains_key(&hostname) {
            // Already have data? Maybe refresh? For now, if we have it, skip or always fetch?
            // Let's always fetch to be safe or maybe check if we want to cache.
            // The instructions say "if the antivirus name contains Sophos...".
            // Implementation: Always fetch for now as this is called via user action or specific criteria.
        }

        if let Some(client) = &self.sophos_client {
            let client = client.clone();
            let t_id = tenant_id.clone();
            let h_name = hostname.clone();

            // Set loading
            self.sophos_loading.insert(hostname.clone(), true);

            tokio::spawn(async move {
                let endpoints_result = async {
                    let region = if let Some(r) = data_region {
                        r
                    } else {
                        // We might need to fetch tenant to get region if not passed.
                        // However in the calling code (handle_key_event) we might not have region easily if we don't have variables.
                        // But we plan to look up from variables.
                        let tenant = client.get_tenant(&t_id).await?;
                        tenant.data_region
                    };

                    let endpoints = client.get_endpoints(&t_id, &region, &h_name).await?;
                    Ok(endpoints)
                }
                .await
                .map_err(|e: anyhow::Error| e.to_string());

                tx.send(Event::SophosEndpointsFetched(h_name, endpoints_result))
                    .unwrap();
            });
        }
    }

    fn fetch_datto_av_agent(
        &mut self,
        hostname: String,
        udf: Option<crate::api::datto::types::Udf>,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.datto_av_client {
            let client = client.clone();
            let h_name = hostname.clone();

            // Check UDF 30 for ID
            let agent_id = udf.as_ref().and_then(|u| u.udf30.clone());

            self.datto_av_loading.insert(hostname.clone(), true);

            tokio::spawn(async move {
                let result = async {
                    if let Some(id) = agent_id {
                        if !id.is_empty() {
                            match client.get_agent_detail(&id).await {
                                Ok(agent) => return Ok(agent),
                                Err(_) => {
                                    // Ignored error (likely ID mismatch or network glitch), falling back to hostname search
                                }
                            }
                        }
                    }
                    // Fallback to filter search by hostname
                    let agents = client.get_agent_details(&h_name).await?;
                    // Assuming we want the first match if any
                    agents
                        .into_iter()
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("No agent found"))
                }
                .await
                .map_err(|e| e.to_string());

                tx.send(Event::DattoAvAgentFetched(h_name, result)).unwrap();
            });
        }
    }

    fn fetch_datto_av_alerts(
        &self,
        agent_id: String,
        hostname: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.datto_av_client {
            let client = client.clone();
            tokio::spawn(async move {
                let result = client
                    .get_agent_alerts(&agent_id)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::DattoAvAlertsFetched(hostname, result))
                    .unwrap();
            });
        }
    }

    fn fetch_datto_av_policies(
        &self,
        agent_id: String,
        hostname: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.datto_av_client {
            let client = client.clone();
            tokio::spawn(async move {
                let result = client
                    .get_agent_policies(&agent_id)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::DattoAvPoliciesFetched(hostname, result))
                    .unwrap();
            });
        }
    }

    #[allow(dead_code)]
    fn scan_datto_av_agent(
        &mut self,
        agent_id: String,
        hostname: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(client) = &self.datto_av_client {
            self.scan_status
                .insert(hostname.clone(), crate::event::ScanStatus::Starting);
            let client = client.clone();
            tokio::spawn(async move {
                let result = client
                    .scan_agent(&agent_id)
                    .await
                    .map_err(|e| e.to_string());
                tx.send(Event::DattoAvScanStarted(hostname, result))
                    .unwrap();
            });
        }
    }

    #[allow(dead_code)]
    fn scan_sophos_endpoint(
        &mut self,
        endpoint_id: String,
        hostname: String,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        if let Some(device) = &self.selected_device {
            // We need tenant ID and region.
            if let Some(site) = self.sites.iter().find(|s| s.uid == device.site_uid) {
                if let Some(vars) = &site.variables {
                    if let Some(id_var) = vars.iter().find(|v| v.name == "tuiMdrId") {
                        let region = vars
                            .iter()
                            .find(|v| v.name == "tuiMdrRegion")
                            .map(|v| v.value.clone());

                        if let Some(client) = &self.sophos_client {
                            let client = client.clone();
                            let t_id = id_var.value.clone();
                            self.scan_status
                                .insert(hostname.clone(), crate::event::ScanStatus::Starting);

                            tokio::spawn(async move {
                                let result = async {
                                    let region = if let Some(r) = region {
                                        r
                                    } else {
                                        let tenant = client.get_tenant(&t_id).await?;
                                        tenant.data_region
                                    };
                                    client.start_scan(&t_id, &region, &endpoint_id).await
                                }
                                .await
                                .map_err(|e| e.to_string());

                                tx.send(Event::SophosScanStarted(hostname, result)).unwrap();
                            });
                        }
                    }
                }
            }
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
        
        // Handle Run Component Input
        if self.show_run_component {
            self.handle_run_component_input(key, tx);
            return;
        }

        // Handle Device Search Input
        if self.show_device_search {
            self.handle_device_search_input(key, tx);
            return;
        }

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
                    } else if let Some(_) = self.editing_udf_index {
                        // UDF Submit
                        self.submit_device_udf(tx);
                    } else {
                        // Variable Submit
                        self.submit_variable(tx);
                    }
                    self.input_state.mode = InputMode::Normal;
                }
                KeyCode::Tab => {
                    // Switch field
                    // Only switch if NOT editing a UDF (UDFs are single value only)
                    if self.editing_udf_index.is_none() {
                        self.input_state.active_field = match self.input_state.active_field {
                            InputField::Name => InputField::Value,
                            InputField::Value => InputField::Name,
                            // No tab switching for simple single-field settings edits for now, keep it simple
                            _ => self.input_state.active_field,
                        };
                    }
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

        match key.code {
            KeyCode::Char('/') => {
                self.show_device_search = true;
                // Don't clear query if we want to remember last search?
                // User said "first, I want a popup search...".
                // Usually search clears or selects all. Let's clear for now.
                self.device_search_query.clear();
                self.device_search_results.clear();
                self.last_search_input = None;
                self.last_searched_query.clear();
                self.device_search_error = None;
                return;
            }
            _ => {}
        }

        match self.current_view {
            CurrentView::List => match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
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
                KeyCode::Enter if self.detail_tab == SiteDetailTab::Devices => {
                    if let Some(idx) = self.devices_table_state.selected() {
                        // Clone device to release borrow on self.devices
                        let device_clone = self.devices.get(idx).cloned();

                        if let Some(device) = device_clone {
                            self.selected_device = Some(device.clone());
                            self.current_view = CurrentView::DeviceDetail;

                            // Auto-load Security Data
                            let is_sophos = device
                                .antivirus
                                .as_ref()
                                .and_then(|av| av.antivirus_product.as_ref())
                                .map(|prod| prod.to_lowercase().contains("sophos"))
                                .unwrap_or(false);

                            let is_datto = device
                                .antivirus
                                .as_ref()
                                .and_then(|av| av.antivirus_product.as_ref())
                                .map(|prod| prod.to_lowercase().contains("datto"))
                                .unwrap_or(false);

                            if is_sophos {
                                // Find site variables for tuiMdrId
                                let sophos_params = if let Some(site) =
                                    self.sites.iter().find(|s| s.uid == device.site_uid)
                                {
                                    if let Some(vars) = &site.variables {
                                        if let Some(id_var) =
                                            vars.iter().find(|v| v.name == "tuiMdrId")
                                        {
                                            let region = vars
                                                .iter()
                                                .find(|v| v.name == "tuiMdrRegion")
                                                .map(|v| v.value.clone());
                                            Some((id_var.value.clone(), region))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                if let Some((id, region)) = sophos_params {
                                    self.fetch_sophos_endpoint(
                                        id,
                                        region,
                                        device.hostname.clone(),
                                        tx.clone(),
                                    );
                                }
                            }

                            if is_datto {
                                self.fetch_datto_av_agent(
                                    device.hostname.clone(),
                                    device.udf.clone(),
                                    tx.clone(),
                                );
                            }

                            // Always fetch activities when entering device detail
                            self.fetch_activity_logs(
                                device.uid.clone(),
                                device.id,
                                device.site_id,
                                tx.clone(),
                            );
                            
                            // Fetch open alerts
                            self.fetch_open_alerts(device.uid.clone(), tx.clone());
                        }
                    }
                }
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
            CurrentView::DeviceDetail => {
                if self.show_device_variables {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('v') | KeyCode::Char('q') => {
                            self.show_device_variables = false;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            let next = match self.udf_table_state.selected() {
                                Some(i) => {
                                    if i >= 29 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            self.udf_table_state.select(Some(next));
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            let next = match self.udf_table_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        29
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            self.udf_table_state.select(Some(next));
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            self.open_edit_udf_modal();
                        }
                        _ => {}
                    }
                    return;
                }

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        // Clear scan loading state for this device if needed
                        if let Some(device) = &self.selected_device {
                            self.scan_status.remove(&device.hostname);
                        }
                        self.current_view = CurrentView::Detail;
                        self.selected_device = None;
                        // Reset tab to default when leaving? Or keep state? Resetting is safer for now.
                        self.device_detail_tab = DeviceDetailTab::OpenAlerts;
                    }
                    KeyCode::Tab => {
                        self.device_detail_tab = match self.device_detail_tab {
                            DeviceDetailTab::OpenAlerts => DeviceDetailTab::Activities,
                            DeviceDetailTab::Activities => DeviceDetailTab::OpenAlerts,
                        };
                    }
                    KeyCode::Char('v') => {
                        self.show_device_variables = true;
                        if self.udf_table_state.selected().is_none() {
                            self.udf_table_state.select(Some(0));
                        }
                    }
                    KeyCode::Char('r') => {
                        self.show_run_component = true;
                        self.run_component_step = RunComponentStep::Search;
                        self.component_search_query.clear();
                        self.fetch_components(tx.clone());
                    }
                    KeyCode::Char('j') | KeyCode::Down => match self.device_detail_tab {
                        DeviceDetailTab::Activities => self.next_activity_log(),
                        DeviceDetailTab::OpenAlerts => self.next_open_alert(),
                    },
                    KeyCode::Char('k') | KeyCode::Up => match self.device_detail_tab {
                        DeviceDetailTab::Activities => self.prev_activity_log(),
                        DeviceDetailTab::OpenAlerts => self.prev_open_alert(),
                    },
                    KeyCode::Enter | KeyCode::Char(' ') => match self.device_detail_tab {
                        DeviceDetailTab::Activities => {
                            if let Some(idx) = self.activity_logs_table_state.selected() {
                                if let Some(log) = self.activity_logs.get(idx) {
                                    self.selected_activity_log = Some(log.clone());
                                    self.current_view = CurrentView::ActivityDetail;

                                    // Parse job ID from details and fetch job result
                                    if let Some(details) = &log.details {
                                        if let Ok(parsed) =
                                            serde_json::from_str::<serde_json::Value>(details)
                                        {
                                            if let Some(job_uid) =
                                                parsed.get("job.uid").and_then(|v| v.as_str())
                                            {
                                                if let Some(device) = &self.selected_device {
                                                    self.fetch_job_result(
                                                        job_uid.to_string(),
                                                        device.uid.clone(),
                                                        tx.clone(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        DeviceDetailTab::OpenAlerts => {
                            // Currently no detailed view for open alerts, but could be added later
                        }
                    },
                    _ => {}
                }
            }
            CurrentView::ActivityDetail => {
                if self.show_popup {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.show_popup = false;
                        }
                        _ => {}
                    }
                    return;
                }

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.current_view = CurrentView::DeviceDetail;
                        self.selected_activity_log = None;
                        self.selected_job_result = None;
                        self.job_result_error = None;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if let Some(job_result) = &self.selected_job_result {
                            let rows = generate_job_rows(job_result);
                            if !rows.is_empty() && self.selected_job_row_index < rows.len() - 1 {
                                self.selected_job_row_index += 1;
                            }
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if self.selected_job_row_index > 0 {
                            self.selected_job_row_index -= 1;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(job_result) = &self.selected_job_result {
                            let rows = generate_job_rows(job_result);
                            if let Some(row) = rows.get(self.selected_job_row_index) {
                                match row {
                                    JobViewRow::StdOutLink(_) => {
                                        if let Some(job_uid) = &job_result.job_uid {
                                            if let Some(device_uid) = &job_result.device_uid {
                                                self.fetch_job_stdout(
                                                    job_uid.clone(),
                                                    device_uid.clone(),
                                                    tx.clone(),
                                                );
                                            }
                                        }
                                    }
                                    JobViewRow::StdErrLink(_) => {
                                        if let Some(job_uid) = &job_result.job_uid {
                                            if let Some(device_uid) = &job_result.device_uid {
                                                self.fetch_job_stderr(
                                                    job_uid.clone(),
                                                    device_uid.clone(),
                                                    tx.clone(),
                                                );
                                            }
                                        }
                                    }
                                    _ => {} // Do nothing for header selection
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
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

    pub fn open_edit_udf_modal(&mut self) {
        if let Some(device) = &self.selected_device {
            if let Some(idx) = self.udf_table_state.selected() {
                // Get current value
                let val = if let Some(udf) = &device.udf {
                    match idx {
                        0 => udf.udf1.clone(),
                        1 => udf.udf2.clone(),
                        2 => udf.udf3.clone(),
                        3 => udf.udf4.clone(),
                        4 => udf.udf5.clone(),
                        5 => udf.udf6.clone(),
                        6 => udf.udf7.clone(),
                        7 => udf.udf8.clone(),
                        8 => udf.udf9.clone(),
                        9 => udf.udf10.clone(),
                        10 => udf.udf11.clone(),
                        11 => udf.udf12.clone(),
                        12 => udf.udf13.clone(),
                        13 => udf.udf14.clone(),
                        14 => udf.udf15.clone(),
                        15 => udf.udf16.clone(),
                        16 => udf.udf17.clone(),
                        17 => udf.udf18.clone(),
                        18 => udf.udf19.clone(),
                        19 => udf.udf20.clone(),
                        20 => udf.udf21.clone(),
                        21 => udf.udf22.clone(),
                        22 => udf.udf23.clone(),
                        23 => udf.udf24.clone(),
                        24 => udf.udf25.clone(),
                        25 => udf.udf26.clone(),
                        26 => udf.udf27.clone(),
                        27 => udf.udf28.clone(),
                        28 => udf.udf29.clone(),
                        29 => udf.udf30.clone(),
                        _ => None,
                    }
                } else {
                    None
                };

                self.input_state = InputState {
                    mode: InputMode::Editing,
                    name_buffer: format!("UDF {}", idx + 1), // Using name buffer for Label display
                    value_buffer: val.unwrap_or_default(),
                    active_field: InputField::Value, // Start on Value
                    is_creating: false,
                    editing_variable_id: None,
                    editing_setting: None,
                };
                self.editing_udf_index = Some(idx);
            }
        }
    }

    pub fn submit_device_udf(&mut self, _tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        if let Some(mut device) = self.selected_device.take() {
            if let Some(idx) = self.editing_udf_index {
                let new_val = self.input_state.value_buffer.clone();
                // Update local device UDF
                let mut udf = device.udf.clone().unwrap_or(crate::api::datto::types::Udf {
                    udf1: None,
                    udf2: None,
                    udf3: None,
                    udf4: None,
                    udf5: None,
                    udf6: None,
                    udf7: None,
                    udf8: None,
                    udf9: None,
                    udf10: None,
                    udf11: None,
                    udf12: None,
                    udf13: None,
                    udf14: None,
                    udf15: None,
                    udf16: None,
                    udf17: None,
                    udf18: None,
                    udf19: None,
                    udf20: None,
                    udf21: None,
                    udf22: None,
                    udf23: None,
                    udf24: None,
                    udf25: None,
                    udf26: None,
                    udf27: None,
                    udf28: None,
                    udf29: None,
                    udf30: None,
                });

                let val_opt = Some(new_val.clone());

                // Update specific field
                match idx {
                    0 => udf.udf1 = val_opt,
                    1 => udf.udf2 = val_opt,
                    2 => udf.udf3 = val_opt,
                    3 => udf.udf4 = val_opt,
                    4 => udf.udf5 = val_opt,
                    5 => udf.udf6 = val_opt,
                    6 => udf.udf7 = val_opt,
                    7 => udf.udf8 = val_opt,
                    8 => udf.udf9 = val_opt,
                    9 => udf.udf10 = val_opt,
                    10 => udf.udf11 = val_opt,
                    11 => udf.udf12 = val_opt,
                    12 => udf.udf13 = val_opt,
                    13 => udf.udf14 = val_opt,
                    14 => udf.udf15 = val_opt,
                    15 => udf.udf16 = val_opt,
                    16 => udf.udf17 = val_opt,
                    17 => udf.udf18 = val_opt,
                    18 => udf.udf19 = val_opt,
                    19 => udf.udf20 = val_opt,
                    20 => udf.udf21 = val_opt,
                    21 => udf.udf22 = val_opt,
                    22 => udf.udf23 = val_opt,
                    23 => udf.udf24 = val_opt,
                    24 => udf.udf25 = val_opt,
                    25 => udf.udf26 = val_opt,
                    26 => udf.udf27 = val_opt,
                    27 => udf.udf28 = val_opt,
                    28 => udf.udf29 = val_opt,
                    29 => udf.udf30 = val_opt,
                    _ => {}
                }

                device.udf = Some(udf.clone());
                self.selected_device = Some(device.clone()); // Restore with updated value locally
                self.editing_udf_index = None;

                // API Call
                if let Some(client) = self.client.clone() {
                    let device_uid = device.uid.clone();
                    tokio::spawn(async move {
                        // Ignoring result for now as per previous pattern or log to stderr
                        if let Err(e) = client.update_device_udf(&device_uid, &udf).await {
                            eprintln!("Failed to update UDF: {}", e);
                        }
                    });
                }
            } else {
                self.selected_device = Some(device); // Restore
            }
        }
    }

    fn next_open_alert(&mut self) {
        let i = match self.open_alerts_table_state.selected() {
            Some(i) => {
                if i >= self.open_alerts.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.open_alerts_table_state.select(Some(i));
    }

    fn prev_open_alert(&mut self) {
        let i = match self.open_alerts_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.open_alerts.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.open_alerts_table_state.select(Some(i));
    }

    fn next_activity_log(&mut self) {
        let i = match self.activity_logs_table_state.selected() {
            Some(i) => {
                if i >= self.activity_logs.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.activity_logs_table_state.select(Some(i));
    }

    fn prev_activity_log(&mut self) {
        let i = match self.activity_logs_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.activity_logs.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.activity_logs_table_state.select(Some(i));
    }

    fn handle_device_search_input(
        &mut self,
        key: KeyEvent,
        tx: tokio::sync::mpsc::UnboundedSender<Event>,
    ) {
        match key.code {
            KeyCode::Esc => {
                self.show_device_search = false;
            }
            KeyCode::Enter => {
                // Select device
                if let Some(idx) = self.device_search_table_state.selected() {
                    if let Some(device) = self.device_search_results.get(idx).cloned() {
                        self.selected_device = Some(device.clone());
                        self.current_view = CurrentView::DeviceDetail;
                        self.show_device_search = false;

                        // Trigger side effects like fetching security data
                         let is_sophos = device
                            .antivirus
                            .as_ref()
                            .and_then(|av| av.antivirus_product.as_ref())
                            .map(|prod| prod.to_lowercase().contains("sophos"))
                            .unwrap_or(false);

                        let is_datto = device
                            .antivirus
                            .as_ref()
                            .and_then(|av| av.antivirus_product.as_ref())
                            .map(|prod| prod.to_lowercase().contains("datto"))
                            .unwrap_or(false);

                        if is_sophos {
                            let sophos_params = if let Some(site) =
                                self.sites.iter().find(|s| s.uid == device.site_uid)
                            {
                                if let Some(vars) = &site.variables {
                                    if let Some(id_var) =
                                        vars.iter().find(|v| v.name == "tuiMdrId")
                                    {
                                        let region = vars
                                            .iter()
                                            .find(|v| v.name == "tuiMdrRegion")
                                            .map(|v| v.value.clone());
                                        Some((id_var.value.clone(), region))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some((id, region)) = sophos_params {
                                self.fetch_sophos_endpoint(
                                    id,
                                    region,
                                    device.hostname.clone(),
                                    tx.clone(),
                                );
                            }
                        }

                        if is_datto {
                            self.fetch_datto_av_agent(
                                device.hostname.clone(),
                                device.udf.clone(),
                                tx.clone(),
                            );
                        }

                        // Always fetch activities when entering device detail
                        self.fetch_activity_logs(
                            device.uid.clone(),
                            device.id,
                            device.site_id,
                            tx.clone(),
                        );
                        
                        // Fetch open alerts
                        self.fetch_open_alerts(device.uid.clone(), tx.clone());
                    }
                }
            }
            KeyCode::Char(c) => {
                self.device_search_query.push(c);
                self.last_search_input = Some(std::time::Instant::now());
            }
            KeyCode::Backspace => {
                self.device_search_query.pop();
                self.last_search_input = Some(std::time::Instant::now());
            }
            KeyCode::Down | KeyCode::Tab => {
                let i = match self.device_search_table_state.selected() {
                    Some(i) => {
                        if i >= self.device_search_results.len().saturating_sub(1) {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.device_search_table_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::BackTab => {
                let i = match self.device_search_table_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.device_search_results.len().saturating_sub(1)
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.device_search_table_state.select(Some(i));
            }
            _ => {}
        }
    }
}
