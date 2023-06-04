use self::utils::{create_dir_in_path, path_to_string};
use anyhow::Result;
use colored::Colorize;
use config::Config;
use fs_extra::{copy_items, dir::CopyOptions};
use slugify::slugify;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::time::Instant;
use walkdir::WalkDir;

mod base;
mod render;
mod settings;
pub mod utils;

pub const PAGES_DIR: &str = "pages";
pub const PUBLIC_DIR: &str = "public";
pub const OUTPUT_DIR: &str = "_site";

pub struct Worker {
    pages_dir: String,
    public_dir: String,
    output_dir: String,
    styles_file: String,
    config_file: String,
}

impl Worker {
    pub fn new(input_dir: &PathBuf) -> Self {
        let pages_dir = path_to_string(&input_dir.join(PAGES_DIR));
        let public_dir = path_to_string(&input_dir.join(PUBLIC_DIR));
        let output_dir = OUTPUT_DIR;

        let syles_file = input_dir
            .join("global.css")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let config_file = input_dir
            .join("Settings.toml")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        Self {
            pages_dir: pages_dir.to_string(),
            public_dir: public_dir.to_string(),
            output_dir: output_dir.to_string(),
            config_file: config_file.to_string(),
            styles_file: syles_file.to_string(),
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
        let settings: settings::Settings = Config::builder()
            .add_source(config::File::with_name(&self.config_file))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();

        // println!("{:#?}", settings);

        settings
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

        for file in &markdown_files {
            let html =
                render::Render::new(&file, &self.styles_file, self.get_settings()).render()?;

            let html_file = file
                .replace(&self.pages_dir, &self.output_dir)
                .replace("page.md", "index.html");

            let html_file = if html_file.contains(".md") {
                let html_file = html_file
                    .replace(".md", "/index.html")
                    .split("/")
                    .map(|x| {
                        if x.contains("index") {
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

            let folder = Path::new(&html_file).parent().unwrap();
            let _ = fs::create_dir_all(folder);
            fs::write(&html_file, html)?;
        }

        let elapsed_time = start_time.elapsed();
        println!("- Completed in: {:?}", elapsed_time);

        Ok(())
    }
}
