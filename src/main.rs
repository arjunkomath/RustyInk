use std::env;
use std::path::PathBuf;
use std::{net::SocketAddr, thread, time::Duration};

use crate::builder::bootstrap;
use crate::builder::utils::path_to_string;
use crate::builder::Worker;
use anyhow::{Ok, Result};
use axum::Router;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

mod builder;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

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
                        project_dir.to_str().unwrap().blue().bold()
                    );
                }
            }
        }
        Commands::Dev { input_dir, watch } => {
            if path_to_string(&input_dir) == path_to_string(&env::current_dir().unwrap()) {
                println!(
                    "{}",
                    "\nSorry, you cannot use current directory as input directory as output is written to it!"
                        .red()
                        .bold()
                );

                return Ok(());
            }

            let worker = Worker::new(&input_dir);
            let output_dir = worker.get_output_dir().to_string();
            let port = worker.get_settings().dev.port.clone();

            // Trigger a build
            match worker.build() {
                Err(e) => {
                    println!("- Build failed -> {}", e.to_string().red().bold());
                }
                _ => {}
            }

            if watch {
                tokio::task::spawn_blocking(move || {
                    let mut hotwatch =
                        hotwatch::Hotwatch::new().expect("hotwatch failed to initialize!");

                    println!(
                        "- Watching for changes in -> {}",
                        input_dir.to_str().unwrap().blue().bold()
                    );
                    hotwatch
                        .watch(input_dir, move |_| {
                            println!("\n- {}", "File(s) changed".bold().yellow());

                            // Rebuild on changes
                            match worker.build() {
                                Err(e) => {
                                    println!("- Build failed -> {}", e.to_string().red().bold());
                                }
                                _ => {}
                            }
                        })
                        .expect("failed to watch content folder!");

                    loop {
                        thread::sleep(Duration::from_secs(1));
                    }
                });
            }

            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            println!(
                "\n- Dev server started on -> {}:{}",
                "http://localhost".bold(),
                port
            );

            let _ = tokio::net::TcpListener::bind(addr).await.unwrap();
            let app = Router::new().nest_service("/", ServeDir::new(output_dir));
            axum::Server::bind(&addr)
                .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
                .await
                .unwrap();
        }
        Commands::Build { input_dir } => {
            let worker = Worker::new(&input_dir);

            match worker.build() {
                Err(e) => {
                    println!("- Build failed -> {}", e.to_string().red().bold());
                }
                _ => {}
            }
        }
    }

    Ok(())
}
