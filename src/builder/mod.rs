use self::utils::{create_dir_in_path, path_to_string};
use anyhow::Result;
use config::Config;
use fs_extra::{copy_items, dir::CopyOptions};
use minify_html::{minify, Cfg};
use owo_colors::OwoColorize;
use slugify::slugify;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::time::Instant;
use walkdir::WalkDir;

pub mod bootstrap;
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
}

impl Worker {
    pub fn new(input_dir: &PathBuf) -> Self {
        let output_dir = OUTPUT_DIR;
        let pages_dir = path_to_string(&input_dir.join(PAGES_DIR));
        let public_dir = path_to_string(&input_dir.join(PUBLIC_DIR));
        let theme_dir = path_to_string(&input_dir.join(THEME_DIR));
        let config_file = path_to_string(&input_dir.join("Settings.toml"));

        Self {
            output_dir: output_dir.to_string(),
            pages_dir: pages_dir.to_string(),
            public_dir: public_dir.to_string(),
            theme_dir: theme_dir.to_string(),
            config_file: config_file.to_string(),
        }
    }

    fn setup_output(&self) -> Result<()> {
        let _ = fs::remove_dir_all(&self.output_dir);
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

        let mut all_file_paths: Vec<String> = Vec::with_capacity(markdown_files.len());

        for file in &markdown_files {
            let html =
                render::Render::new(&file, &self.theme_dir, self.get_settings()).render_page()?;

            let html_file = file
                .replace(&self.pages_dir, &self.output_dir)
                .replace("page.md", "index.html");

            let html_file = if html_file.contains(".md") {
                let html_file = html_file
                    .replace(".md", "/index.html")
                    .split("/")
                    .map(|x| {
                        if x.contains("index") || x == OUTPUT_DIR {
                            x.to_string()
                        } else {
                            format!("{}", slugify!(x))
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("/");
                html_file
            } else {
                html_file
            };

            let folder = Path::new(&html_file).parent().unwrap_or_else(|| {
                println!(
                    "{}: {}",
                    "Failed to create folder for generated file, ".red(),
                    &html_file
                );
                std::process::exit(1);
            });
            let _ = fs::create_dir_all(folder);

            let minified = minify(&html.as_bytes(), &Cfg::new());

            fs::write(&html_file, minified)?;
            all_file_paths.push(html_file);
        }

        // Handle robots.txt, ignore if there is a file already
        if !Path::new(&self.output_dir).join("robots.txt").exists() {
            if let Ok(robots_txt) = seo::generate_robots_txt(&self.get_settings()) {
                fs::write(Path::new(&self.output_dir).join("robots.txt"), robots_txt)?;
            }
        }

        // Handle sitemap.xml, ignore if there is a file already
        if !Path::new(&self.output_dir).join("sitemap.xml").exists() {
            if let Ok(sitemap_xml) =
                seo::generate_sitemap_xml(&self.get_settings(), &self.output_dir, &all_file_paths)
            {
                if let Some(sitemap_xml) = sitemap_xml {
                    fs::write(Path::new(&self.output_dir).join("sitemap.xml"), sitemap_xml)?;
                }
            }
        }

        let elapsed_time = start_time.elapsed();
        println!("✔ Completed in: {:?}", elapsed_time);

        Ok(())
    }
}
