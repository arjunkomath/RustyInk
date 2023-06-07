use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use super::utils::create_dir_in_path;
use anyhow::Result;
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
    project_dir: &PathBuf,
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
    headers.insert(USER_AGENT, "RustyInk".parse().unwrap());

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
            let file_name = item.name;
            let file_path = project_dir.join(item.path.clone());

            let folder = Path::new(&file_path).parent().unwrap();
            let _ = fs::create_dir_all(folder);

            let file = format!("{}/{}", folder.to_str().unwrap(), file_name);
            println!("Downloading theme file: {}", file);

            let download_url = format!(
                "https://raw.githubusercontent.com/{}/{}/master/{}",
                owner, repo, item.path
            );

            let mut file_response = client.get(download_url).send().await?;
            let mut file = fs::File::create(file)?;
            while let Some(chunk) = file_response.chunk().await? {
                file.write_all(&chunk)?;
            }
        } else if item.is_dir() {
            let dir_name = item.name;
            let subfolder_path = format!("{}/{}", folder_path, dir_name);

            download_folder(project_dir, client, owner, repo, &subfolder_path).await?;
        }
    }

    Ok(())
}

pub async fn download_theme(project_dir: &PathBuf, theme: &str) -> Result<()> {
    create_dir_in_path(&project_dir)?;

    let client = Client::new();
    let repo_owner = "arjunkomath";
    let repo_name = "rustyink-themes";

    println!("- Downloading theme {}", theme.bold().blue());

    download_folder(project_dir, &client, repo_owner, repo_name, theme).await?;

    Ok(())
}
