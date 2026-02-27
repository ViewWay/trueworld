// crates/server/src/main.rs

#![allow(clippy::too_many_arguments)]

mod config;
mod database;
mod entity;
mod game;
mod network;
mod player;
mod room;
mod server;
mod shutdown;

use std::time::Duration;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{EnvFilter, fmt};

use server::TrueWorldServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    init_logging();

    info!("TrueWorld Server starting...");

    // 加载配置
    let config = config::ServerConfig::load()?;

    // 创建并运行服务器
    let server = TrueWorldServer::new(config).await?;
    server.run().await?;

    Ok(())
}

fn init_logging() {
    fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("trueworld=debug,info")),
        )
        .init();
}
