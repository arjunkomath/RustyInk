use std::{
    format, fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::shared::{logger::Logger, settings::Settings, utils};

use anyhow::{Context, Result};
use async_recursion::async_recursion;
use owo_colors::OwoColorize;
use reqwest::{
    header::{HeaderMap, USER_AGENT},
    Client,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
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

#[async_recursion]
async fn download_folder(
    project_dir: &str,
    theme: &str,
    client: &Client,
    owner: &str,
    repo: &str,
    folder_path: &str,
) -> Result<()> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        owner, repo, folder_path
    );

    // Create a custom user agent header to avoid rate limiting
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "RustyInk".parse()?);

    // Send the API request to get the folder contents
    let response = client
        .get(&url)
        .headers(headers)
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

            Logger::new().activity(&format!("\tDownloading file: {}", item.path.bold().green()));

            let download_url = format!(
                "https://raw.githubusercontent.com/{}/{}/master/{}",
                owner, repo, item.path
            );

            let mut file_response = client.get(download_url).send().await?;
            let mut file = fs::File::create(file_path)?;
            while let Some(chunk) = file_response.chunk().await? {
                file.write_all(&chunk)?;
            }
        } else if item.is_dir() {
            let dir_name = item.name;
            let subfolder_path = format!("{}/{}", folder_path, dir_name);

            download_folder(project_dir, theme, client, owner, repo, &subfolder_path).await?;
        }
    }

    Ok(())
}

pub async fn project(project_dir: &PathBuf, theme: &str) -> Result<()> {
    utils::create_dir_in_path(project_dir)?;

    let project_dir = utils::path_to_string(project_dir)?;

    let client = Client::new();
    let repo_owner = "arjunkomath";
    let repo_name = "rustyink-themes";

    Logger::new().activity(&format!("Downloading theme {}", theme.bold().blue()));

    download_folder(&project_dir, theme, &client, repo_owner, repo_name, theme).await?;

    Ok(())
}

pub fn settings_file(project_dir: &PathBuf) -> Result<()> {
    let settings = Settings::default().to_toml_string()?;
    fs::write(Path::new(project_dir).join("Settings.toml"), settings)?;

    Ok(())
}
