use std::{net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use owo_colors::OwoColorize;
use tokio::sync::broadcast;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{builder::Worker, shared::logger::Logger};

pub const WEBSOCKET_CLIENT_JS: &str = r#"
    <script type="module">
        const socket = new WebSocket(`ws://${location.host}/__livereload`);

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

pub async fn start(output_dir: String, port: u16, reload_tx: broadcast::Sender<()>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = std::net::TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;

    let app = Router::new()
        .route("/__livereload", get(livereload_handler))
        .nest_service("/", ServeDir::new(output_dir))
        .with_state(reload_tx);

    Logger::new().success(&format!(
        "Dev server started on -> http://localhost:{}",
        port.blue()
    ));

    axum::Server::from_tcp(listener)?
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await?;

    Ok(())
}

async fn livereload_handler(
    ws: WebSocketUpgrade,
    State(reload_tx): State<broadcast::Sender<()>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, reload_tx.subscribe()))
}

async fn handle_socket(socket: WebSocket, mut reload_rx: broadcast::Receiver<()>) {
    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            reload = reload_rx.recv() => match reload {
                Ok(()) => {
                    if sender
                        .send(Message::Text("RELOAD".to_string()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            },
            incoming = receiver.next() => match incoming {
                Some(Ok(_)) => continue,
                // Client disconnected or errored, end this connection task
                _ => break,
            },
        }
    }
}

pub fn handle_file_changes(
    input_dir: PathBuf,
    worker: Worker,
    reload_tx: broadcast::Sender<()>,
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
                    // A send error only means no browser tabs are connected
                    let _ = reload_tx.send(());
                }
            }
        }
    }

    Ok(())
}
