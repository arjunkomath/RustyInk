use std::fs;

use super::{
    settings::{self, Link},
    utils::{insert_kv_into_yaml, parse_string_to_yaml},
};
use anyhow::{Context, Result};
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

impl Render {
    pub fn new(file: &str, theme_dir: &str, settings: settings::Settings) -> Self {
        Self {
            file: file.to_string(),
            theme_dir: theme_dir.to_string(),
            settings: settings.clone(),
        }
    }

    pub fn render_page(&self, site_directory: &serde_yaml::Value) -> Result<String> {
        let (metadata, markdown) = self.get_markdown_and_metadata()?;

        let content = if let Some(metadata) = metadata {
            let metadata = parse_string_to_yaml(&metadata)?;

            let content = self
                .render_body(&markdown, &metadata, site_directory)
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

    pub fn get_markdown_and_metadata(&self) -> Result<(Option<String>, String)> {
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
                &metadata,
                "body",
                &serde_yaml::Value::String(body.to_string()),
            )?;
            let metadata = insert_kv_into_yaml(&metadata, "root", &site_directory[""])?;

            println!("{}", serde_json::to_string_pretty(&metadata)?);

            let body =
                Handlebars::new().render_template(&self.get_template(&template)?, &metadata)?;

            Ok(body)
        } else {
            Ok(body.to_string())
        };

        template
    }
}
