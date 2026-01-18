pub mod api;
pub mod app;
pub mod config;
pub mod event;
pub mod tui;
pub mod ui;

use anyhow::Result;
use api::datto::DattoClient;
use api::datto_av::DattoAvClient;
use api::sophos::SophosClient;
use app::App;
use config::Config;
use event::EventHandler;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    // Initialize API Client
    let mut client = DattoClient::new(config.datto).expect("Failed to create API client");
    let rocket_client = crate::api::rocket_cyber::RocketCyberClient::new(config.rocket).ok(); // Create Rocket client
    let sophos_client = SophosClient::new(config.sophos).ok(); // Create Sophos client
    let datto_av_client = DattoAvClient::new(config.datto_av).ok(); // Create Datto AV client

    // Authenticate
    if let Err(e) = client.authenticate().await {
        eprintln!("Warning: Authentication failed: {}", e);
    }

    // Setup terminal
    let mut terminal = tui::init()?;
    tui::install_panic_hook();

    // Create app and event handler including tick rate
    // Create app and event handler including tick rate
    let mut app = App::new(Some(client), rocket_client, sophos_client, datto_av_client);

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
