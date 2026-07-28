use std::{
    format, fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::shared::{logger::Logger, settings::Settings, utils};

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use reqwest::Client;

#[derive(Debug, serde::Deserialize)]
struct GitHubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    r#type: String,
}

impl GitHubContent {
    fn is_file(&self) -> bool {
        self.r#type == "file"
    }

    fn is_dir(&self) -> bool {
        self.r#type == "dir"
    }
}

async fn download_folder(
    project_dir: &str,
    theme: &str,
    client: &Client,
    owner: &str,
    repo: &str,
    folder_path: &str,
) -> Result<()> {
    let mut folders = vec![folder_path.to_string()];

    while let Some(folder_path) = folders.pop() {
        let response = client
            .get(format!(
                "https://api.github.com/repos/{owner}/{repo}/contents/{folder_path}"
            ))
            .send()
            .await?
            .json::<Vec<GitHubContent>>()
            .await?;

        for item in response {
            if item.is_file() {
                let file_path = item.path.replace(theme, project_dir);
                let folder = Path::new(&file_path)
                    .parent()
                    .context("Failed to get parent folder")?;
                fs::create_dir_all(folder)?;

                Logger::activity(&format!("\tDownloading file: {}", item.path.bold().green()));

                let mut response = client
                    .get(format!(
                        "https://raw.githubusercontent.com/{owner}/{repo}/master/{}",
                        item.path
                    ))
                    .send()
                    .await?;
                let mut file = fs::File::create(file_path)?;
                while let Some(chunk) = response.chunk().await? {
                    file.write_all(&chunk)?;
                }
            } else if item.is_dir() {
                folders.push(format!("{folder_path}/{}", item.name));
            }
        }
    }

    Ok(())
}

pub async fn project(project_dir: &PathBuf, theme: &str) -> Result<()> {
    fs::create_dir_all(project_dir)?;

    let project_dir = utils::path_to_string(project_dir)?;

    let client = Client::builder().user_agent("RustyInk").build()?;
    let repo_owner = "arjunkomath";
    let repo_name = "rustyink-themes";

    Logger::activity(&format!("Downloading theme {}", theme.bold().blue()));

    download_folder(&project_dir, theme, &client, repo_owner, repo_name, theme).await?;

    Ok(())
}

pub fn settings_file(project_dir: &PathBuf) -> Result<()> {
    let settings = toml::to_string(&Settings::default())?;
    fs::write(Path::new(project_dir).join("Settings.toml"), settings)?;

    Ok(())
}
