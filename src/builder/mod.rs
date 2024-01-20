use crate::{
    dev::server::WEBSOCKET_CLIENT_JS,
    shared::{settings, utils},
};

use anyhow::{Context, Result};
use config::Config;
use fs_extra::dir::CopyOptions;
use html_minifier::HTMLMinifier;
use owo_colors::OwoColorize;
use rayon::prelude::*;
use slugify::slugify;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::time::Instant;
use walkdir::WalkDir;

pub mod cache;
mod handlebar_helpers;
mod render;
mod seo;

pub const PAGES_DIR: &str = "pages";
pub const PUBLIC_DIR: &str = "public";
pub const THEME_DIR: &str = "theme";
pub const OUTPUT_DIR: &str = "_site";

pub struct Worker {
    pages_dir: String,
    public_dir: String,
    theme_dir: String,
    output_dir: String,
    config_file: String,
    cache: Option<cache::Cache>,
    is_dev: bool,
}

impl Worker {
    pub fn dev(input_dir: &Path, cache: Option<cache::Cache>, is_dev: bool) -> Result<Self> {
        let output_dir = OUTPUT_DIR;
        let pages_dir = utils::path_to_string(&input_dir.join(PAGES_DIR))?;
        let public_dir = utils::path_to_string(&input_dir.join(PUBLIC_DIR))?;
        let theme_dir = utils::path_to_string(&input_dir.join(THEME_DIR))?;
        let config_file = utils::path_to_string(&input_dir.join("Settings.toml"))?;

        utils::create_dir_in_path(&PathBuf::from(output_dir))?;

        Ok(Self {
            output_dir: output_dir.to_string(),
            pages_dir,
            public_dir,
            theme_dir,
            config_file,
            cache,
            is_dev,
        })
    }

    pub fn prod(input_dir: &Path) -> Result<Self> {
        let output_dir = OUTPUT_DIR;
        let pages_dir = utils::path_to_string(&input_dir.join(PAGES_DIR))?;
        let public_dir = utils::path_to_string(&input_dir.join(PUBLIC_DIR))?;
        let theme_dir = utils::path_to_string(&input_dir.join(THEME_DIR))?;
        let config_file = utils::path_to_string(&input_dir.join("Settings.toml"))?;

        utils::create_dir_in_path(&PathBuf::from(output_dir))?;

        Ok(Self {
            output_dir: output_dir.to_string(),
            pages_dir,
            public_dir,
            theme_dir,
            config_file,
            cache: None,
            is_dev: false,
        })
    }

    fn setup_output(&self) -> Result<()> {
        fs::remove_dir_all(&self.output_dir)?;
        utils::create_dir_in_path(&PathBuf::from(&self.output_dir))?;

        Ok(())
    }

    fn copy_public_files(&self) -> Result<()> {
        let public_files: Vec<String> = WalkDir::new(&self.public_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().display().to_string())
            .skip(1)
            .collect();
        let options = CopyOptions::new();
        fs_extra::copy_items(&public_files, &self.output_dir, &options)?;

        Ok(())
    }

    pub fn get_output_dir(&self) -> &str {
        &self.output_dir
    }

    pub fn get_settings(&self) -> settings::Settings {
        match Config::builder()
            .add_source(config::File::with_name(&self.config_file))
            .build()
        {
            Ok(config) => match config.try_deserialize() {
                Ok(settings) => settings,
                Err(e) => {
                    println!("{}: {}", "Failed to parse settings file, ".red(), e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                println!("{}: {}", "Failed to open settings file, ".red(), e);
                std::process::exit(1);
            }
        }
    }

    pub fn build(&self) -> Result<()> {
        println!("{}...", "\n- Building site".bold());

        let start_time = Instant::now();

        self.setup_output()?;
        self.copy_public_files()?;

        let markdown_files: Vec<String> = WalkDir::new(&self.pages_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().display().to_string().ends_with(".md"))
            .map(|e| e.path().display().to_string())
            .collect();

        // Used for generating site directory
        let all_pages_with_metadata: Vec<(String, String)> = markdown_files
            .par_iter()
            .map(|x| {
                let metadata = render::Render::new(
                    x,
                    &self.theme_dir,
                    self.get_settings(),
                    self.cache.clone(),
                )
                .get_metadata()
                .unwrap_or(None);

                let x = x.replace(&self.pages_dir, "").replace("page.md", "");

                let x = if x.contains(".md") {
                    let x = x
                        .replace(".md", "")
                        .split('/')
                        .map(|x| slugify!(x))
                        .collect::<Vec<String>>()
                        .join("/");
                    x
                } else {
                    x
                };

                (x, metadata.unwrap_or("".to_string()))
            })
            .filter(|x| x.0 != "/" || x.1.is_empty())
            .collect();

        let site_directory = self.generate_site_directory(&all_pages_with_metadata)?;

        markdown_files.par_iter().for_each(|file| {
            if let Err(e) = self.process_file(file, &site_directory) {
                println!("{}: {}", "Failed to process file, ".red(), e);
            }
        });

        // Handle robots.txt, ignore if there is a file already
        if !Path::new(&self.output_dir).join("robots.txt").exists() {
            if let Ok(robots_txt) = seo::generate_robots_txt(&self.get_settings()) {
                println!(
                    "{} {} robots.txt",
                    "✔ Generated".green(),
                    "File       ".blue()
                );
                fs::write(Path::new(&self.output_dir).join("robots.txt"), robots_txt)?;
            }
        }

        // Handle sitemap.xml, ignore if there is a file already
        if !Path::new(&self.output_dir).join("sitemap.xml").exists() {
            if let Ok(sitemap_xml) =
                seo::generate_sitemap_xml(&self.get_settings(), &all_pages_with_metadata)
            {
                println!(
                    "{} {} sitemap.xml",
                    "✔ Generated".green(),
                    "File       ".blue()
                );
                fs::write(Path::new(&self.output_dir).join("sitemap.xml"), sitemap_xml)?;
            }
        }

        let elapsed_time = start_time.elapsed();
        println!("✔ Completed in: {:?}", elapsed_time);

        Ok(())
    }

    fn process_file(&self, file: &str, site_directory: &serde_yaml::Value) -> Result<()> {
        let html_file = file
            .replace(&self.pages_dir, &self.output_dir)
            .replace("page.md", "index.html");

        let html_file = if html_file.contains(".md") {
            let html_file = html_file
                .replace(".md", "/index.html")
                .split('/')
                .map(|x| {
                    if x.contains("index") || x == OUTPUT_DIR {
                        x.to_string()
                    } else {
                        slugify!(x)
                    }
                })
                .collect::<Vec<String>>()
                .join("/");
            html_file
        } else {
            html_file
        };

        let actual_url_path = html_file
            .replace(&self.output_dir, "")
            .replace("index.html", "");

        let html = render::Render::new(
            file,
            &self.theme_dir,
            self.get_settings(),
            self.cache.clone(),
        )
        .render_page("app", &actual_url_path, site_directory)?;

        let folder = Path::new(&html_file)
            .parent()
            .context("Failed to get parent folder")?;
        fs::create_dir_all(folder)?;

        println!(
            "{} {} {}",
            "✔ Generated".green(),
            "Page       ".blue(),
            &html_file
        );

        if self.is_dev {
            // Add websocket client to html
            let html = format!("{}\n{}", html, WEBSOCKET_CLIENT_JS);
            fs::write(&html_file, html)?;
            return Ok(());
        }

        let mut html_minifier = HTMLMinifier::new();
        html_minifier.digest(&html)?;
        fs::write(&html_file, html_minifier.get_html())?;

        // Handle AMP
        let amp = render::Render::new(
            file,
            &self.theme_dir,
            self.get_settings(),
            self.cache.clone(),
        )
        .render_page("amp", &actual_url_path, site_directory)?;

        if amp.is_empty() {
            return Ok(());
        }

        let amp_file = html_file.replace("index.html", "amp/index.html");
        println!(
            "{} {} {}",
            "✔ Generated".green(),
            "AMP        ".blue(),
            &amp_file
        );
        let amp_folder = Path::new(&amp_file)
            .parent()
            .context("Failed to get parent folder")?;
        fs::create_dir_all(amp_folder)?;
        fs::write(amp_file, amp)?;

        Ok(())
    }

    pub fn generate_site_directory(
        &self,
        url_paths: &Vec<(String, String)>,
    ) -> Result<serde_yaml::Value> {
        let mut yaml = serde_yaml::Mapping::new();

        for (url_path, metadata) in url_paths {
            let mut current_yaml = &mut yaml;

            let mut url_path = url_path.split('/').collect::<Vec<&str>>();
            let last = url_path.pop().unwrap_or("_self");
            let last = if last.is_empty() { "_self" } else { last };

            for path in url_path {
                if path.is_empty() {
                    continue;
                }

                if !current_yaml.contains_key(&serde_yaml::Value::String(path.to_string())) {
                    current_yaml.insert(
                        serde_yaml::Value::String(path.to_string()),
                        serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
                    );
                }

                current_yaml =
                    match current_yaml.get_mut(&serde_yaml::Value::String(path.to_string())) {
                        Some(serde_yaml::Value::Mapping(x)) => x,
                        _ => {
                            println!("{}: {}", "Failed to parse yaml".red(), "Invalid yaml".red());
                            std::process::exit(1);
                        }
                    };
            }

            if let Ok(metadata) = utils::parse_string_to_yaml(metadata) {
                current_yaml.insert(serde_yaml::Value::String(last.to_string()), metadata);
            } else {
                current_yaml.insert(
                    serde_yaml::Value::String(last.to_string()),
                    serde_yaml::Value::Null,
                );
            }
        }

        Ok(serde_yaml::Value::Mapping(yaml))
    }
}
