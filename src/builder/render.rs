use std::fs;

use crate::builder::utils::download_url_as_string;

use super::{
    cache,
    seo::generate_open_graph_tags,
    settings::{self, Link},
    utils::{insert_kv_into_yaml, parse_string_to_yaml},
};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct Render {
    file: String,
    theme_dir: String,
    settings: settings::Settings,
    cache: Option<cache::Cache>,
}

#[derive(Serialize, Deserialize)]
struct RenderData {
    title: String,
    description: String,
    open_graph_tags: String,
    styles: String,
    scripts: String,

    links: Vec<Link>,
    content: String,
    page_metadata: Option<serde_yaml::Value>,
}

impl Render {
    pub fn new(
        file: &str,
        theme_dir: &str,
        settings: settings::Settings,
        cache: Option<cache::Cache>,
    ) -> Self {
        Self {
            file: file.to_string(),
            theme_dir: theme_dir.to_string(),
            settings,
            cache,
        }
    }

    pub fn render_page(&self, site_directory: &serde_yaml::Value) -> Result<String> {
        let (metadata, markdown) = self.get_markdown_and_metadata()?;

        let metadata = if let Some(metadata) = metadata {
            let metadata = parse_string_to_yaml(&metadata)?;
            Some(metadata)
        } else {
            None
        };

        let content = if let Some(metadata) = &metadata {
            self.render_body(&markdown, metadata, site_directory)
                .with_context(|| format!("Failed to render page: {}", self.file))?
        } else {
            markdown
        };

        let html = Handlebars::new().render_template(
            &self.get_template("app")?,
            &RenderData {
                title: self.settings.meta.title.clone(),
                description: self.settings.meta.description.clone(),
                open_graph_tags: generate_open_graph_tags(&self.settings)?,
                content,
                styles: self.get_global_styles()?,
                scripts: self.get_global_scripts()?,
                links: self.settings.navigation.links.clone(),
                page_metadata: metadata,
            },
        )?;

        Ok(html)
    }

    pub fn get_metadata(&self) -> Result<Option<String>> {
        let markdown = fs::read_to_string(&self.file)?;

        let metadata = Regex::new(r"^(?s)---(.*?)---")
            .context("Failed to parse metadata from markdown file")?;

        if let Some(captures) = metadata.captures(&markdown) {
            let metadata = captures
                .get(1)
                .with_context(|| format!("Failed to get metadata from captures: {}", self.file))?
                .as_str();

            Ok(Some(metadata.to_string()))
        } else {
            Ok(None)
        }
    }

    fn get_template(&self, name: &str) -> Result<String> {
        let template = fs::read_to_string(format!("{}/{}.hbs", self.theme_dir, name))?;

        Ok(template)
    }

    fn get_global_styles(&self) -> Result<String> {
        let styles = fs::read_to_string(format!("{}/global.css", self.theme_dir))?;

        let downloaded_styles = self
            .settings
            .get_site_settings()
            .get_style_urls()
            .par_iter()
            .map(
                |url| match download_url_as_string(url, self.cache.clone()) {
                    Ok(style) => style,
                    Err(e) => {
                        println!("Failed to download style: {}", e);
                        String::new()
                    }
                },
            )
            .collect::<Vec<String>>()
            .join("\n");

        Ok(downloaded_styles + "\n" + styles.as_str())
    }

    fn get_global_scripts(&self) -> Result<String> {
        let downloaded_scripts = self
            .settings
            .get_site_settings()
            .get_script_urls()
            .par_iter()
            .map(
                |url| match download_url_as_string(url, self.cache.clone()) {
                    Ok(script) => script,
                    Err(e) => {
                        println!("Failed to download script: {}", e);
                        String::new()
                    }
                },
            )
            .collect::<Vec<String>>()
            .join("\n");

        Ok(downloaded_scripts)
    }

    fn get_markdown_and_metadata(&self) -> Result<(Option<String>, String)> {
        let markdown = fs::read_to_string(&self.file)?;

        let metadata = Regex::new(r"^(?s)---(.*?)---(.*)")
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

            let parser = pulldown_cmark::Parser::new_ext(markdown, pulldown_cmark::Options::all());
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

    fn render_body(
        &self,
        body: &str,
        metadata: &serde_yaml::Value,
        site_directory: &serde_yaml::Value,
    ) -> Result<String> {
        let template = if let Some(template) = metadata.get("template") {
            let template = template
                .as_str()
                .with_context(|| format!("Failed to get template from metadata: {}", self.file))?;

            let metadata = insert_kv_into_yaml(
                metadata,
                "body",
                &serde_yaml::Value::String(body.to_string()),
            )?;
            let metadata = insert_kv_into_yaml(&metadata, "root", site_directory)?;

            // println!("{}", serde_json::to_string_pretty(&metadata)?);

            let body =
                Handlebars::new().render_template(&self.get_template(template)?, &metadata)?;

            Ok(body)
        } else {
            Ok(body.to_string())
        };

        template
    }
}
