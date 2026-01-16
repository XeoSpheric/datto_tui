pub mod api;
pub mod app;
pub mod config;
pub mod event;
pub mod tui;
pub mod ui;

use anyhow::Result;
use api::RmmClient;
use app::App;
use config::Config;
use event::EventHandler;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = Config::from_env();
    let mut client = match config {
        Ok(cfg) => Some(RmmClient::new(cfg)?),
        Err(e) => {
            eprintln!("Warning: Failed to load config: {}", e);
            None
        }
    };

    if let Some(ref mut c) = client {
        if let Err(e) = c.authenticate().await {
            eprintln!("Warning: Authentication failed: {}", e);
            // We continue, but the app will likely fail to fetch data.
            // The UI should ideally show this error, but for now we print to stderr before TUI init.
        }
    }

    // Setup terminal
    let mut terminal = tui::init()?;
    tui::install_panic_hook();

    // Create app and event handler including tick rate
    let mut app = App::new(client);
    let tick_rate = Duration::from_millis(250);
    let mut events = EventHandler::new(tick_rate);

    // Run the app (async)
    let res = app.run(&mut terminal, &mut events).await;

    // Restore terminal
    tui::restore()?;

    // Print error if any
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
