use crate::api::datto::types::{ActivityLogsResponse, DevicesResponse, JobResult, SitesResponse};
use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    SitesFetched(Result<SitesResponse, String>),
    DevicesFetched(Result<DevicesResponse, String>),
    IncidentsFetched(Result<Vec<crate::api::rocket_cyber::types::Incident>, String>),
    SiteVariablesFetched(
        String,
        Result<Vec<crate::api::datto::types::SiteVariable>, String>,
    ), // (Site UID, Result)
    VariableCreated(
        String,
        Result<crate::api::datto::types::SiteVariable, String>,
    ),
    VariableUpdated(
        String,
        Result<crate::api::datto::types::SiteVariable, String>,
    ),
    SiteUpdated(Result<crate::api::datto::types::Site, String>),
    SophosCasesFetched(String, Result<Vec<crate::api::sophos::Case>, String>),
    SophosEndpointsFetched(String, Result<Vec<crate::api::sophos::Endpoint>, String>), // (Hostname, Result)
    SophosScanStarted(String, Result<(), String>), // (Hostname, Result)
    DattoAvAgentFetched(
        String,
        Result<crate::api::datto_av::types::AgentDetail, String>,
    ), // (Hostname, Result)
    DattoAvScanStarted(String, Result<(), String>), // (Hostname, Result)
    ScanStatusChanged(String, ScanStatus),
    DattoAvAlertsFetched(
        String,
        Result<Vec<crate::api::datto_av::types::Alert>, String>,
    ),
    DattoAvPoliciesFetched(String, Result<serde_json::Value, String>),
    DeviceSearchResultsFetched(Result<DevicesResponse, String>),
    ActivityLogsFetched(Result<ActivityLogsResponse, String>),
    OpenAlertsFetched(String, Result<Vec<crate::api::datto::types::Alert>, String>), // (DeviceUID, Result)
    JobResultFetched(Result<JobResult, String>),
    JobStdOutFetched(Result<Vec<crate::api::datto::types::JobStdOutput>, String>),
    JobStdErrFetched(Result<Vec<crate::api::datto::types::JobStdOutput>, String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScanStatus {
    Starting,
    Started,
}

#[derive(Debug)]
pub struct EventHandler {
    _tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
    _task: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: std::time::Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();
        let task_tx = tx.clone();
        let _task = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                let tick_delay = interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    _ = tick_delay => {
                        task_tx.send(Event::Tick).unwrap();
                    }
                    Some(Ok(evt)) = crossterm_event => {
                        match evt {
                            CrosstermEvent::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                    task_tx.send(Event::Key(key)).unwrap();
                                }
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                task_tx.send(Event::Mouse(mouse)).unwrap();
                            }
                            CrosstermEvent::Resize(w, h) => {
                                task_tx.send(Event::Resize(w, h)).unwrap();
                            }
                            _ => {}
                        }
                    }
                };
            }
        });
        Self { _tx, rx, _task }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self._tx.clone()
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Unable to get event"))
    }
}
