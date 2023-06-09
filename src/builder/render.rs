use std::fs;

use super::settings::{self, Link};
use anyhow::{Context, Result};
use config::Config;
use handlebars::Handlebars;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct Render {
    pub file: String,
    pub theme_dir: String,
    pub settings: settings::Settings,
}

#[derive(Serialize, Deserialize)]
struct RenderData {
    meta_title: String,
    meta_description: String,
    styles: String,
    links: Vec<Link>,
    code_highlighting: bool,

    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    pub template: Option<String>,

    pub title: Option<String>,
    pub footnote: Option<String>,
    pub author: Option<String>,
    pub author_link: Option<String>,
    pub date_published: Option<String>,

    pub body: Option<String>,
}

impl PageMetadata {
    pub fn from_yaml_string(metadata: String) -> Result<Self> {
        let metadata: PageMetadata = Config::builder()
            .add_source(config::File::from_str(&metadata, config::FileFormat::Yaml))
            .build()
            .context("Failed to parse metadata")
            .and_then(|r| r.try_deserialize().context("Failed to parse metadata"))?;

        Ok(Self { ..metadata })
    }
}

impl Render {
    pub fn new(file: &str, theme_dir: &str, settings: settings::Settings) -> Self {
        Self {
            file: file.to_string(),
            theme_dir: theme_dir.to_string(),
            settings: settings.clone(),
        }
    }

    pub fn render_page(&self) -> Result<String> {
        let (metadata, markdown) = self.get_markdown_and_metadata()?;

        let content = if let Some(metadata) = metadata {
            let metadata: PageMetadata = PageMetadata::from_yaml_string(metadata)?;

            let content = self
                .render_body(&markdown, metadata)
                .with_context(|| format!("Failed to render page: {}", self.file))?;

            content
        } else {
            markdown
        };

        let styles = self.get_global_styles()?;

        let html = Handlebars::new().render_template(
            &self.get_template("app")?,
            &RenderData {
                meta_title: self.settings.meta.title.clone(),
                meta_description: self.settings.meta.description.clone(),
                content,
                styles,
                links: self.settings.navigation.links.clone(),
                code_highlighting: self
                    .settings
                    .get_site_settings()
                    .is_code_highlighting_enabled(),
            },
        )?;

        Ok(html)
    }

    fn get_template(&self, name: &str) -> Result<String> {
        let template = fs::read_to_string(format!("{}/{}.hbs", self.theme_dir, name))?;

        Ok(template)
    }

    fn get_global_styles(&self) -> Result<String> {
        let styles = fs::read_to_string(format!("{}/global.css", self.theme_dir))?;

        Ok(styles)
    }

    fn get_markdown_and_metadata(&self) -> Result<(Option<String>, String)> {
        let markdown = fs::read_to_string(&self.file)?;

        let metadata = Regex::new(r"(?s)---(.*?)---(.*)")
            .context("Failed to parse metadata from markdown file")?;

        if let Some(captures) = metadata.captures(&markdown) {
            let metadata = captures
                .get(1)
                .with_context(|| format!("Failed to get metadata from captures: {}", self.file))?
                .as_str();
            let markdown = captures
                .get(2)
                .with_context(|| format!("Failed to get markdown from captures: {}", self.file))?
                .as_str();

            let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());
            let mut content = String::new();
            pulldown_cmark::html::push_html(&mut content, parser);

            Ok((Some(metadata.to_string()), content))
        } else {
            let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());
            let mut content = String::new();
            pulldown_cmark::html::push_html(&mut content, parser);

            Ok((None, content))
        }
    }

    fn render_body(&self, body: &str, metadata: PageMetadata) -> Result<String> {
        let template = metadata
            .template
            .clone()
            .context("Failed to get template")?;

        if template.is_empty() {
            Ok(body.to_string())
        } else {
            let body = Handlebars::new().render_template(
                &self.get_template(&template)?,
                &PageMetadata {
                    body: Some(body.to_string()),
                    ..metadata
                },
            )?;
            Ok(body)
        }
    }
}
