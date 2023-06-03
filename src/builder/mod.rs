use anyhow::Result;
use fs_extra::{copy_items, dir::CopyOptions};
use std::{fs, path::Path};
use walkdir::WalkDir;

mod base;

pub const OUTPUT_DIR: &str = "output";

fn create_dir(dir: &str) -> Result<()> {
    if let Err(err) = fs::metadata(&dir) {
        if err.kind() == std::io::ErrorKind::NotFound {
            if let Err(err) = fs::create_dir_all(&dir) {
                eprintln!("Failed to create pages directory: {}", err);
            }
        }
    }

    Ok(())
}

pub struct Worker {
    pages_dir: String,
    public_dir: String,
}

impl Worker {
    pub fn new(pages_dir: &str, public_dir: &str) -> Self {
        create_dir(pages_dir).unwrap();
        create_dir(public_dir).unwrap();

        Self {
            pages_dir: pages_dir.to_string(),
            public_dir: public_dir.to_string(),
        }
    }

    fn setup_output(&self) -> Result<()> {
        let _ = fs::remove_dir_all(&OUTPUT_DIR);
        create_dir(OUTPUT_DIR).unwrap();

        Ok(())
    }

    fn copy_public_files(&self) -> Result<()> {
        let public_files: Vec<String> = WalkDir::new(&self.public_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().display().to_string())
            .skip(1)
            .collect();
        println!("Copying public files...");
        let options = CopyOptions::new();
        copy_items(&public_files, OUTPUT_DIR, &options)?;

        Ok(())
    }

    pub fn rebuild(&self) -> Result<()> {
        println!("Rebuilding site...");

        self.setup_output()?;
        self.copy_public_files()?;

        // Handle pages
        let markdown_files: Vec<String> = WalkDir::new(&self.pages_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().display().to_string().ends_with(".md"))
            .map(|e| e.path().display().to_string())
            .collect();

        let mut html_files = Vec::with_capacity(markdown_files.len());

        for file in &markdown_files {
            println!("Processing file: {}", file);

            let mut html = base::HEADER.to_owned();

            let markdown = fs::read_to_string(&file)?;
            let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());
            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, parser);

            html.push_str(base::render_body(&body).as_str());
            html.push_str(base::FOOTER);

            let html_file = file
                .replace(&self.pages_dir, OUTPUT_DIR)
                .replace("page.md", "index.html");

            let folder = Path::new(&html_file).parent().unwrap();

            let _ = fs::create_dir_all(folder);
            fs::write(&html_file, html)?;

            html_files.push(html_file);
        }

        Ok(())
    }
}
