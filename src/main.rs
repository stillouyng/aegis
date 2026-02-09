use std::net::SocketAddr;
use tokio::sync::mpsc;

pub mod core;
pub mod protocols;

use crate::core::events::Event;
use crate::protocols::tcp::listener::TcpByteListener;
use crate::protocols::tcp::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Set up the address.
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    // 2. Event channel.
    let (tx, mut rx) = mpsc::channel(100);

    // 3. Transport initialization
    let tcp_listener = TcpByteListener::bind(addr).await?;
    let listener = Box::new(tcp_listener);

    println!("Server starting on {}", addr);

    // 4. Start the server
    tokio::spawn(async move {
        if let Err(e) = server::run_server(listener, tx).await {
            eprintln!("Server Error: {}", e);
        }
    });

    // 5. App logic
    println!("Event loop started...");
    while let Some(event) = rx.recv().await {
        match event {
            Event::RequestStart { meta, .. } => {
                println!(
                    "New Request: {:?} (Content-Length: {:?})",
                    meta.content_type, meta.content_length
                );
            }
            Event::RequestBody { more_body, .. } => {
                if !more_body {
                    println!("Body fully received");
                }
            }
            Event::Disconnect => {
                println!("Client disconnected");
            }
        }
    }

    Ok(())
}
