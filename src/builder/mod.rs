use self::utils::{create_dir_in_path, parse_string_to_yaml, path_to_string};
use anyhow::{Context, Result};
use config::Config;
use fs_extra::{copy_items, dir::CopyOptions};
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

pub mod bootstrap;
pub mod cache;
mod render;
mod seo;
pub mod settings;
pub mod utils;

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
}

impl Worker {
    pub fn new(input_dir: &Path, cache: Option<cache::Cache>) -> Result<Self> {
        let output_dir = OUTPUT_DIR;
        let pages_dir = path_to_string(&input_dir.join(PAGES_DIR))?;
        let public_dir = path_to_string(&input_dir.join(PUBLIC_DIR))?;
        let theme_dir = path_to_string(&input_dir.join(THEME_DIR))?;
        let config_file = path_to_string(&input_dir.join("Settings.toml"))?;

        create_dir_in_path(&PathBuf::from(output_dir))?;

        Ok(Self {
            output_dir: output_dir.to_string(),
            pages_dir,
            public_dir,
            theme_dir,
            config_file,
            cache,
        })
    }

    fn setup_output(&self) -> Result<()> {
        fs::remove_dir_all(&self.output_dir)?;
        create_dir_in_path(&PathBuf::from(&self.output_dir))?;

        Ok(())
    }

    fn copy_public_files(&self) -> Result<()> {
        let public_files: Vec<String> = WalkDir::new(&self.public_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().display().to_string())
            .skip(1)
            .collect();
        let options = CopyOptions::new();
        copy_items(&public_files, &self.output_dir, &options)?;

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
                println!("{} robots.txt", "✔ Generated".green());
                fs::write(Path::new(&self.output_dir).join("robots.txt"), robots_txt)?;
            }
        }

        // Handle sitemap.xml, ignore if there is a file already
        if !Path::new(&self.output_dir).join("sitemap.xml").exists() {
            if let Ok(sitemap_xml) =
                seo::generate_sitemap_xml(&self.get_settings(), &all_pages_with_metadata)
            {
                println!("{} sitemap.xml", "✔ Generated".green());
                fs::write(Path::new(&self.output_dir).join("sitemap.xml"), sitemap_xml)?;
            }
        }

        let elapsed_time = start_time.elapsed();
        println!("✔ Completed in: {:?}", elapsed_time);

        Ok(())
    }

    fn process_file(&self, file: &str, site_directory: &serde_yaml::Value) -> Result<()> {
        let html = render::Render::new(
            file,
            &self.theme_dir,
            self.get_settings(),
            self.cache.clone(),
        )
        .render_page(site_directory)?;

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

        let folder = Path::new(&html_file)
            .parent()
            .context("Failed to get parent folder")?;
        fs::create_dir_all(folder)?;

        println!("{} {}", "✔ Generated".green(), &html_file);

        let mut html_minifier = HTMLMinifier::new();
        html_minifier.digest(&html)?;
        fs::write(&html_file, html_minifier.get_html())?;

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

            if let Ok(metadata) = parse_string_to_yaml(metadata) {
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
