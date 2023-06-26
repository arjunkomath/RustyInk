use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{env, println};

use crate::builder::bootstrap;
use crate::builder::utils::path_to_string;
use crate::builder::Worker;
use anyhow::{Context, Result};
use builder::{cache, utils};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use owo_colors::OwoColorize;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, tungstenite, WebSocketStream};

mod builder;

type Clients = Arc<Mutex<HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>>>;

#[derive(Debug, Parser)]
#[command(name = "rustyink")]
#[command(bin_name = "rustyink")]
#[command(about = "A blazing fast static site generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create new project
    #[command()]
    New {
        #[clap(required = true, help = "Project directory")]
        project_dir: PathBuf,

        #[clap(
            required = false,
            help = "Theme",
            default_value = "pico",
            short = 't',
            long = "theme"
        )]
        theme: String,
    },
    /// Start dev mode
    #[command()]
    Dev {
        #[clap(required = true, help = "Input directory")]
        input_dir: PathBuf,

        /// Watch for changes
        #[clap(short = 'w', long = "watch")]
        watch: bool,
    },
    /// Build the site
    #[command()]
    Build {
        #[clap(required = true, help = "Input directory")]
        input_dir: PathBuf,
    },
    /// Clean the site
    #[command()]
    Clean {},
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let project_dirs =
        ProjectDirs::from("rs", "cli", "RustyInk").context("Failed to get project directories")?;
    let cache_dir = project_dirs.cache_dir().to_string_lossy().to_string();
    let cache = cache::Cache::new(cache_dir)?;

    match args.command {
        Commands::New { project_dir, theme } => {
            println!("{}...", "\n- Creating new project".bold());

            match bootstrap::download_theme(&project_dir, &theme).await {
                Err(e) => {
                    println!("- {}", e.to_string().red().bold());
                }
                _ => {
                    println!(
                        "- Project created in {}",
                        project_dir.display().blue().bold()
                    );
                }
            }
        }
        Commands::Dev { input_dir, watch } => {
            if path_to_string(&input_dir)? == path_to_string(&env::current_dir()?)? {
                println!(
                    "{}",
                    "\nSorry, you cannot use current directory as input directory as output is written to it!"
                        .red()
                        .bold()
                );

                return Ok(());
            }

            let worker = Worker::new(&input_dir, Some(cache))?;
            let output_dir = worker.get_output_dir().to_string();
            let port = worker.get_settings().dev.port;

            // Trigger a build
            if let Err(e) = worker.build() {
                println!("- Build failed -> {}", e.to_string().red().bold());
            }

            // Start dev server
            tokio::task::spawn(utils::start_dev_server(output_dir, port));

            if watch {
                let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

                tokio::spawn(handle_file_changes(input_dir, worker, clients.clone()));

                let addr = "127.0.0.1:3001".to_string();
                println!("✔ Listening socket connections on {}", addr.blue().bold());

                let listener = TcpListener::bind(&addr).await?;

                while let Ok((stream, _)) = listener.accept().await {
                    tokio::spawn(accept_connection(stream, clients.clone()));
                }
            }
        }
        Commands::Build { input_dir } => {
            let worker = Worker::new(&input_dir, None)?;

            if let Err(e) = worker.build() {
                println!("- Build failed -> {}", e.to_string().red().bold());
            }
        }
        Commands::Clean {} => {
            cache.clean().context("Failed to clean cache")?;
        }
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream, clients: Clients) -> Result<()> {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = accept_async(stream).await?;

    let (write, _) = ws_stream.split();

    clients.lock().await.insert(addr.to_string(), write);

    Ok(())
}

async fn handle_file_changes(input_dir: PathBuf, worker: Worker, clients: Clients) -> Result<()> {
    println!(
        "✔ Watching for changes in -> {}",
        input_dir.display().blue().bold()
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx).unwrap();
    debouncer
        .watcher()
        .watch(input_dir.as_path(), RecursiveMode::Recursive)
        .expect("Failed to watch content folder!");

    for result in rx {
        match result {
            Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
            Ok(_) => {
                println!("{}", "\n✔ Changes detected, rebuilding...".cyan());
                /* Ok is not working here for some reason */
                if let Err(e) = worker.build() {
                    println!("- Build failed -> {}", e.to_string().red().bold());
                } else {
                    // Send message to all clients to reload
                    let mut clients = clients.lock().await;
                    for (_, client) in clients.iter_mut() {
                        client
                            .send(tungstenite::Message::Text("reload".to_string()))
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}
