use std::{fs, net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result};
use axum::Router;
use owo_colors::OwoColorize;
use tower_http::{services::ServeDir, trace::TraceLayer};

use super::cache::Cache;

pub fn create_dir_in_path(path: &PathBuf) -> Result<()> {
    fs::create_dir(&path)?;

    Ok(())
}

pub fn path_to_string(path: &PathBuf) -> Result<String> {
    path.canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", path))
        .and_then(|x| {
            x.to_str()
                .context("Failed to parse patht to string")
                .map(|s| s.to_string())
        })
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

pub fn parse_string_to_yaml(string: &str) -> Result<serde_yaml::Value> {
    let metadata: serde_yaml::Value = serde_yaml::from_str(string)?;
    Ok(metadata)
}

pub fn insert_kv_into_yaml(
    yaml: &serde_yaml::Value,
    key: &str,
    value: &serde_yaml::Value,
) -> Result<serde_yaml::Value> {
    let mut yaml = yaml.clone();
    yaml[key] = value.clone();

    Ok(yaml)
}

pub fn download_url_as_string(url: &str, cache: Cache) -> Result<String> {
    if let Some(content) = cache.get(url) {
        return Ok(content.to_string());
    }

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?;
    let content = response.text()?;

    cache.set(url, content.as_str())?;

    Ok(content)
}
