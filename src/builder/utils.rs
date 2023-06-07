use std::{fs, net::SocketAddr, path::PathBuf, process};

use anyhow::Result;
use axum::Router;
use owo_colors::OwoColorize;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn create_dir_in_path(path: &PathBuf) -> Result<()> {
    fs::create_dir(&path)?;

    Ok(())
}

pub fn path_to_string(path: &PathBuf) -> String {
    match path.canonicalize() {
        Ok(path) => match path.to_str() {
            Some(path) => path.to_string(),
            None => {
                println!(
                    "✘ {}",
                    "Error: Path is not a valid UTF-8 sequence".bold().red()
                );
                process::exit(1);
            }
        },
        _ => {
            println!(
                "✘ {}",
                format!(
                    "Error: {}",
                    format!("Path {:?} does not exist", path).bold().red()
                )
                .red()
            );
            process::exit(1);
        }
    }
}

pub async fn start_dev_server(output_dir: String, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let _ = tokio::net::TcpListener::bind(addr).await?;

    let app = Router::new().nest_service("/", ServeDir::new(output_dir));

    println!(
        "✔ Starting Dev server on -> {}:{}",
        "http://localhost".bold(),
        port
    );

    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await?;

    Ok(())
}
