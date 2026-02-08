mod listener;
use listener::listener::run_listener;
use listener::events::Event;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel(100);
    let addr: &str = "127.0.0.1:8080";
    
    // Start the listener
    tokio::spawn(async move {
        if let Err(e) = run_listener(addr, tx).await {
            eprintln!("Error starting listener: {}", e);
        };
    });

    // Consume events
    while let Some(event) = rx.recv().await {
        println!("Received event: {:?}", event);
    };

}
