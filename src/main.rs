use std::fs;
use std::path::PathBuf;
use std::{net::SocketAddr, thread, time::Duration};

use crate::builder::utils::{create_dir_in_path, path_to_string};
use crate::builder::{Worker, PAGES_DIR, PUBLIC_DIR};
use anyhow::{Ok, Result};
use axum::Router;
use builder::settings::Settings;
use clap::{Parser, Subcommand};
use colored::Colorize;
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
        Commands::New { project_dir } => {
            println!("{}...", "\n- Creating new project".bold());

            create_dir_in_path(&project_dir)?;
            create_dir_in_path(&project_dir.join(PAGES_DIR))?;
            create_dir_in_path(&project_dir.join(PUBLIC_DIR))?;

            let project_dir_path = path_to_string(&project_dir);

            let settings = Settings::new();
            let settings_file = format!("{}/Settings.toml", &project_dir_path);
            fs::write(&settings_file, &settings).unwrap();

            let global_css_file = format!("{}/global.css", &project_dir_path);
            let global_css_content = String::from(
                r#"@import url('https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css');"#,
            );
            fs::write(&global_css_file, global_css_content).unwrap();

            let pages_dir_path = path_to_string(&project_dir.join(PAGES_DIR));
            let index_file = format!("{}/page.md", &pages_dir_path);
            let index_content = r#"Hello!"#;
            fs::write(&index_file, index_content).unwrap();

            println!("{}", "- Done!".bold());
        }
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

                    println!(
                        "- Watching for changes in -> {}",
                        input_dir.to_str().unwrap().blue().bold()
                    );
                    hotwatch
                        .watch(input_dir, move |_| {
                            println!("\n- {}", "File(s) changed".bold().yellow());
                            worker.build().unwrap();
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
            worker.build().unwrap();
        }
    }

    Ok(())
}
