use dotenvy::dotenv;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};

pub mod core;
pub mod protocols;

use crate::core::events::Event;
use crate::core::structs::ServerConfig;
use crate::protocols::tcp::listener::TcpByteListener;
use crate::protocols::tcp::server;

fn init_logging() {
    dotenv().ok();
    registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configurate
    init_logging();
    let config = Arc::new(ServerConfig::from_env());
    let span = tracing::info_span!("connection", server = %config.addr);
    let _enter = span.enter();

    // 2. Event channel.
    let (tx, mut rx) = mpsc::channel(100);

    // 3. Transport initialization
    let tcp_listener = TcpByteListener::bind(config.addr).await?;
    let listener = Box::new(tcp_listener);
    let server_config = Arc::clone(&config);

    info!("Server starting");

    // 4. Start the server
    tokio::spawn(async move {
        if let Err(e) = server::run_server(listener, tx, server_config).await {
            error!("Server Error: {}", e);
        }
    });

    // 5. App logic
    info!("Event loop started");
    while let Some(event) = rx.recv().await {
        match event {
            Event::RequestStart { meta, .. } => {
                info!(
                    "New Request: {:?} (Content-Length: {:?})",
                    meta.content_type, meta.content_length
                );
            }
            Event::RequestBody { more_body, .. } => {
                if !more_body {
                    info!("Body fully received");
                }
            }
            Event::Disconnect { client_addr: _ } => {
                info!("Client disconnected");
            }
        }
    }

    Ok(())
}
