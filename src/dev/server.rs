use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use axum::Router;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use owo_colors::OwoColorize;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{self, Message},
    WebSocketStream,
};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{builder::Worker, shared::logger::Logger};

pub type Clients = Arc<Mutex<HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>>>;

pub const WEBSOCKET_CLIENT_JS: &str = r#"
    <script type="module">
        const socket = new WebSocket("ws://localhost:3001");

        socket.onopen = function (event) {
            console.log("✔ Connected to dev server");
        };

        socket.onmessage = function (event) {
            if (event.data === "RELOAD") {
                window.location.reload();
            }
        };
    </script>
"#;

pub async fn start(output_dir: String, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let _ = tokio::net::TcpListener::bind(addr).await?;

    let app = Router::new().nest_service("/", ServeDir::new(output_dir));

    Logger::new().success(&format!(
        "Dev server started on -> http://localhost:{}",
        port.blue()
    ));

    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await?;

    Ok(())
}

pub async fn accept_connection(stream: TcpStream, clients: Clients) -> Result<()> {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");

    let ws_stream = accept_async(stream).await?;

    let (write, _) = ws_stream.split();

    clients.lock().await.insert(addr.to_string(), write);

    Ok(())
}

pub async fn handle_file_changes(
    input_dir: PathBuf,
    worker: Worker,
    clients: Clients,
) -> Result<()> {
    let log = Logger::new();

    log.success(&format!(
        "Watching for changes in -> {}",
        input_dir.display().blue().bold()
    ));

    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), tx)?;
    debouncer
        .watcher()
        .watch(input_dir.as_path(), RecursiveMode::Recursive)?;

    for result in rx {
        match result {
            Err(error) => log.error(&error.to_string()),
            Ok(_) => {
                log.info("\nChanges detected, rebuilding...");

                if let Err(e) = worker.build() {
                    log.error(&format!("Build failed -> {}", e.to_string().red().bold()));
                } else {
                    log.success("Build successful, reloading...");
                    // Send message to all clients to reload
                    let mut clients = clients.lock().await;
                    for (_, client) in clients.iter_mut() {
                        client
                            .send(tungstenite::Message::Text("RELOAD".to_string()))
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}
