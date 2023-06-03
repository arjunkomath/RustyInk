use anyhow::Result;
use config::Config;
use fs_extra::{copy_items, dir::CopyOptions};
use handlebars::Handlebars;
use log::{info, trace};
use slugify::slugify;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::time::Instant;
use walkdir::WalkDir;

mod base;
mod settings;

pub const PAGES_DIR: &str = "pages";
pub const PUBLIC_DIR: &str = "public";
pub const OUTPUT_DIR: &str = "output";

fn create_dir(dir: &str) -> Result<()> {
    if let Err(err) = fs::metadata(&dir) {
        if err.kind() == std::io::ErrorKind::NotFound {
            if let Err(err) = fs::create_dir_all(&dir) {
                eprintln!("Failed to create {} directory: {}", dir, err);
            }
        }
    }

    Ok(())
}

pub struct Worker {
    pages_dir: String,
    public_dir: String,
    output_dir: String,
    settings: settings::Settings,
}

impl Worker {
    pub fn new(input_dir: &PathBuf) -> Self {
        let pages_dir = input_dir
            .join(PAGES_DIR)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let public_dir = input_dir
            .join(PUBLIC_DIR)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let output_dir = OUTPUT_DIR;

        let config_file = input_dir
            .join("Settings.toml")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let settings: settings::Settings = Config::builder()
            .add_source(config::File::with_name(&config_file))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();

        // println!("{:#?}", settings);

        Self {
            pages_dir: pages_dir.to_string(),
            public_dir: public_dir.to_string(),
            output_dir: output_dir.to_string(),
            settings,
        }
    }

    fn setup_output(&self) -> Result<()> {
        let _ = fs::remove_dir_all(&self.output_dir);
        create_dir(&self.output_dir).unwrap();

        Ok(())
    }

    fn copy_public_files(&self) -> Result<()> {
        let public_files: Vec<String> = WalkDir::new(&self.public_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().display().to_string())
            .skip(1)
            .collect();
        info!("Copying public files...");
        let options = CopyOptions::new();
        copy_items(&public_files, &self.output_dir, &options)?;

        Ok(())
    }

    pub fn get_output_dir(&self) -> &str {
        &self.output_dir
    }

    pub fn get_settings(&self) -> &settings::Settings {
        &self.settings
    }

    pub fn build(&self) -> Result<()> {
        info!("Rebuilding site...");

        let start_time = Instant::now();

        self.setup_output()?;
        self.copy_public_files()?;

        // Handle pages
        let markdown_files: Vec<String> = WalkDir::new(&self.pages_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().display().to_string().ends_with(".md"))
            .map(|e| e.path().display().to_string())
            .collect();

        for file in &markdown_files {
            trace!("Processing file: {}", file);

            let mut html = base::HEADER.to_owned();

            let markdown = fs::read_to_string(&file)?;
            let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());
            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, parser);

            html.push_str(&base::render_article(&body).as_str());
            html.push_str(base::FOOTER);

            let reg = Handlebars::new();
            let html = reg.render_template(&html, &self.settings.site)?;

            let top_navigation = base::render_links(&self.settings.site.top_navigation);
            let html = html.replace("%%LINKS%%", &top_navigation);

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
        info!("Completed in: {:?}", elapsed_time);

        Ok(())
    }
}
