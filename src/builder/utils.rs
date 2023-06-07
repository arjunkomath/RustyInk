use std::{fs, net::SocketAddr, path::PathBuf};

use anyhow::Result;
use axum::Router;
use owo_colors::OwoColorize;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn create_dir_in_path(path: &PathBuf) -> Result<()> {
    fs::create_dir(&path)?;

    Ok(())
}

pub fn path_to_string(path: &PathBuf) -> String {
    path.canonicalize().unwrap().to_str().unwrap().to_string()
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
