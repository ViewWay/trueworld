// crates/client/src/main.rs

mod app;
mod state;
mod network;
mod input;
mod render;
mod connection;
mod net_sync;
mod movement;

use app::TrueWorldClient;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let client = TrueWorldClient::new()?;
    client.run();

    Ok(())
}
