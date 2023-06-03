use crate::builder::Worker;
use anyhow::{Ok, Result};
use axum::Router;
use clap::{Parser, Subcommand};
use log::{info, warn};
use std::path::PathBuf;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();

    match args.command {
        Commands::Dev { watch, input_dir } => {
            let worker = Worker::new(&input_dir);
            let output_dir = worker.get_output_dir().to_string();
            let port = worker.get_settings().dev.port.clone();

            // Trigger a build
            worker.build().unwrap();

            if watch {
                tokio::task::spawn_blocking(move || {
                    let mut hotwatch =
                        hotwatch::Hotwatch::new().expect("hotwatch failed to initialize!");

                    info!("Watching for changes in -> {}", input_dir.to_str().unwrap());
                    hotwatch
                        .watch(input_dir, move |_| {
                            worker.build().unwrap();
                        })
                        .expect("failed to watch content folder!");

                    loop {
                        thread::sleep(Duration::from_secs(1));
                    }
                });
            }

            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            warn!("Dev server started on -> http://localhost:{}", port);

            let _ = tokio::net::TcpListener::bind(addr).await.unwrap();
            let app = Router::new().nest_service("/", ServeDir::new(output_dir));
            axum::Server::bind(&addr)
                .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
                .await
                .unwrap();
        }
        Commands::Build { input_dir } => {
            let worker = Worker::new(&input_dir);
            worker.build().unwrap();
        }
    }

    Ok(())
}
