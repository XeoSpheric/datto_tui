use crate::api::RmmClient;
use crate::api::types::{Device, Site};
use crate::event::{Event, EventHandler};
use crate::tui::Tui;
use crate::ui;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;

#[derive(Debug, PartialEq)]
pub enum CurrentView {
    List,
    Detail,
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub counter: u8,
    pub sites: Vec<Site>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub client: Option<RmmClient>,
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
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            counter: 0,
            sites: Vec::new(),
            is_loading: false,
            error: None,
            client: None,
            current_view: CurrentView::List,

            table_state: TableState::default(),
            current_page: 0,
            total_pages: 0,
            total_count: 0,

            devices: Vec::new(),
            devices_loading: false,
            devices_error: None,
            devices_table_state: TableState::default(),
        }
    }
}

impl App {
    pub fn new(client: Option<RmmClient>) -> Self {
        let mut app = Self::default();
        app.client = client;
        app
    }

    pub async fn run(&mut self, tui: &mut Tui, events: &mut EventHandler) -> Result<()> {
        // Initial fetch
        if self.client.is_some() {
            self.fetch_sites(events.sender());
        } else {
            self.error = Some("API Client not initialized. Check .env config.".to_string());
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
            }
        }
        Ok(())
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

    fn handle_key_event(&mut self, key: KeyEvent, tx: tokio::sync::mpsc::UnboundedSender<Event>) {
        match self.current_view {
            CurrentView::List => match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
                KeyCode::Char('n') | KeyCode::Right => self.next_page(tx),
                KeyCode::Char('p') | KeyCode::Left => self.prev_page(tx),
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
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Char('q') => {
                    self.current_view = CurrentView::List;
                }
                KeyCode::Char('j') | KeyCode::Down => self.next_device(),
                KeyCode::Char('k') | KeyCode::Up => self.prev_device(),
                _ => {}
            },
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
}
