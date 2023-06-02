use anyhow::Result;
use std::{fs, path::Path};
use walkdir::WalkDir;

mod templates;

pub const OUTPUT_DIR: &str = "output";

fn create_dir(dir: &str) -> Result<()> {
    if let Err(err) = fs::metadata(&dir) {
        if err.kind() == std::io::ErrorKind::NotFound {
            if let Err(err) = fs::create_dir_all(&dir) {
                eprintln!("Failed to create pages directory: {}", err);
            } else {
                println!("Pages directory created successfully!");
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
        create_dir(OUTPUT_DIR).unwrap();

        Self {
            pages_dir: pages_dir.to_string(),
            public_dir: public_dir.to_string(),
        }
    }

    pub fn rebuild(&self) -> Result<()> {
        println!("Rebuilding site {} {}", self.pages_dir, self.public_dir);

        let _ = fs::remove_dir_all(&OUTPUT_DIR);

        let public_files: Vec<String> = WalkDir::new(&self.public_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().display().to_string())
            .collect();

        for file in &public_files {
            println!("Processing file: {}", file);
        }

        let markdown_files: Vec<String> = WalkDir::new(&self.pages_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().display().to_string().ends_with(".md"))
            .map(|e| e.path().display().to_string())
            .collect();

        let mut html_files = Vec::with_capacity(markdown_files.len());

        for file in &markdown_files {
            println!("Processing file: {}", file);

            let mut html = templates::HEADER.to_owned();

            let markdown = fs::read_to_string(&file)?;
            let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());
            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, parser);

            html.push_str(templates::render_body(&body).as_str());
            html.push_str(templates::FOOTER);

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
