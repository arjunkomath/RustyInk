use crate::builder::{Worker, OUTPUT_DIR};
use anyhow::{Ok, Result};
use axum::Router;
use clap::{Parser, Subcommand};
use log::{info, warn};
use std::{net::SocketAddr, thread, time::Duration};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

mod builder;

#[derive(Debug, Parser)]
#[command(name = "rink")]
#[command(bin_name = "rink")]
#[command(about = "A blazing fast static site generator in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start dev mode
    #[command()]
    Dev {
        /// Watch for changes
        #[clap(short = 'w', long = "watch")]
        watch: bool,
    },
}

const PAGES_DIR: &str = "pages";
const PUBLIC_DIR: &str = "public";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();

    match args.command {
        Commands::Dev { watch } => {
            if watch {
                tokio::task::spawn_blocking(move || {
                    let mut hotwatch =
                        hotwatch::Hotwatch::new().expect("hotwatch failed to initialize!");
                    let worker = Worker::new(PAGES_DIR, PUBLIC_DIR);
                    worker.rebuild().unwrap();

                    info!("Watching for changes in -> {}", PAGES_DIR);
                    hotwatch
                        .watch(PAGES_DIR, move |_| {
                            worker.rebuild().unwrap();
                        })
                        .expect("failed to watch content folder!");

                    loop {
                        thread::sleep(Duration::from_secs(1));
                    }
                });
            }

            let port: u16 = std::env::var("PORT")
                .unwrap_or("3000".to_string())
                .parse()?;
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            warn!("Dev server started on -> http://localhost:{}", port);

            let _ = tokio::net::TcpListener::bind(addr).await.unwrap();
            let app = Router::new().nest_service("/", ServeDir::new(OUTPUT_DIR));
            axum::Server::bind(&addr)
                .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
                .await
                .unwrap();
        }
    }

    Ok(())
}
