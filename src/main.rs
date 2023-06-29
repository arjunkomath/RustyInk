use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, println};

use crate::builder::bootstrap;
use crate::builder::dev_server::Clients;
use crate::builder::Worker;
use anyhow::{Context, Result};
use builder::{cache, dev_server, utils};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

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
                    // Create settings file
                    bootstrap::create_settings_file(&project_dir)?;

                    println!(
                        "- Project created in {}",
                        project_dir.display().blue().bold()
                    );
                }
            }
        }
        Commands::Dev { input_dir, watch } => {
            if utils::path_to_string(&input_dir)? == utils::path_to_string(&env::current_dir()?)? {
                println!(
                    "{}",
                    "\nSorry, you cannot use current directory as input directory as output is written to it!"
                        .red()
                        .bold()
                );

                return Ok(());
            }

            let worker = Worker::new(&input_dir, Some(cache), true)?;
            let output_dir = worker.get_output_dir().to_string();
            let port = worker.get_settings().dev.port;

            // Trigger a build
            if let Err(e) = worker.build() {
                println!("- Build failed -> {}", e.to_string().red().bold());
            }

            // Start dev server
            tokio::task::spawn(dev_server::start_dev_server(output_dir, port));

            if watch {
                let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

                tokio::spawn(dev_server::handle_file_changes(
                    input_dir,
                    worker,
                    clients.clone(),
                ));

                let addr = "0.0.0.0:3001".to_string();
                println!("✔ Listening socket connections on {}", addr.blue().bold());

                let listener = TcpListener::bind(&addr).await?;

                while let Ok((stream, _)) = listener.accept().await {
                    tokio::spawn(dev_server::accept_connection(stream, clients.clone()));
                }
            }
        }
        Commands::Build { input_dir } => {
            let worker = Worker::new(&input_dir, None, true)?;

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
