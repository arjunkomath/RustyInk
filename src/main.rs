use std::env;
use std::path::PathBuf;
use std::{thread, time::Duration};

use crate::builder::bootstrap;
use crate::builder::utils::path_to_string;
use crate::builder::Worker;
use anyhow::{Ok, Result};
use builder::utils;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

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
                        project_dir.display().blue().bold()
                    );
                }
            }
        }
        Commands::Dev { input_dir, watch } => {
            if path_to_string(&input_dir)
                == path_to_string(&env::current_dir().unwrap_or(PathBuf::from(".")))
            {
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
                        "✔ Watching for changes in -> {}",
                        input_dir.display().blue().bold()
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

            match utils::start_dev_server(output_dir, port).await {
                Err(e) => {
                    println!(
                        "- Failed to start dev server: {}",
                        e.to_string().red().bold()
                    );
                }
                _ => {}
            }
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
