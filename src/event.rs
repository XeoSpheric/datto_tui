use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    SitesFetched(Result<crate::api::types::SitesResponse, String>),
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
