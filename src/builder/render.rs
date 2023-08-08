use std::{fs, path::Path};

use super::seo;
use crate::shared::{
    settings::{self, Link},
    utils,
};

use super::{cache, handlebar_helpers};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct Render<'a> {
    file: String,
    theme_dir: String,
    settings: settings::Settings,
    cache: Option<cache::Cache>,
    handlebars: Handlebars<'a>,
}

#[derive(Serialize, Deserialize)]
struct AppRenderData {
    title: String,
    description: String,
    open_graph_tags: String,
    styles: String,
    scripts: String,
    links: Vec<Link>,
    content: String,
    page_metadata: Option<serde_yaml::Value>,
    data: Option<toml::Value>,
    remote_data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct PageRenderData {
    body: String,
    root: serde_yaml::Value,
    data: serde_yaml::Value,
    remote_data: serde_json::Value,
}

impl Render<'_> {
    pub fn new(
        file: &str,
        theme_dir: &str,
        settings: settings::Settings,
        cache: Option<cache::Cache>,
    ) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("slice", Box::new(handlebar_helpers::SliceHelper));
        handlebars.register_helper("stringify", Box::new(handlebar_helpers::StringifyHelper));
        handlebars.register_helper("sort-by", Box::new(handlebar_helpers::SortByHelper));
        handlebars.register_helper(
            "format-date",
            Box::new(handlebar_helpers::DateFormaterHelper),
        );

        Self {
            file: file.to_string(),
            theme_dir: theme_dir.to_string(),
            settings,
            cache,
            handlebars,
        }
    }

    pub fn render_page(&self, site_directory: &serde_yaml::Value) -> Result<String> {
        let (metadata, markdown) = self.get_markdown_and_metadata()?;

        let metadata = if let Some(metadata) = metadata {
            let metadata = utils::parse_string_to_yaml(&metadata)?;
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

        let html = self
            .handlebars
            .render_template(
                &self.get_template("app").context("Failed to get template")?,
                &AppRenderData {
                    title: self.settings.meta.title.clone(),
                    description: self.settings.meta.description.clone(),
                    open_graph_tags: seo::generate_open_graph_tags(&self.settings)?,
                    content,
                    styles: self.get_global_styles()?,
                    scripts: self.get_global_scripts()?,
                    links: self.settings.navigation.links.clone(),
                    page_metadata: metadata,
                    data: self.settings.data.clone(),
                    remote_data: self.get_remote_data()?,
                },
            )
            .context("Failed to render page")?;

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
        let css_path = format!("{}/global.css", self.theme_dir);

        let styles = if Path::new(&css_path).exists() {
            fs::read_to_string(css_path)?
        } else {
            String::new()
        };

        let downloaded_styles = self
            .settings
            .get_site_settings()
            .get_style_urls()
            .par_iter()
            .map(|url| match cache::get_file(url, self.cache.clone()) {
                Ok(style) => style,
                Err(e) => {
                    println!("Failed to download style: {}", e);
                    String::new()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        let style_tag = format!("<style>{}{}</style>", downloaded_styles, styles);

        Ok(style_tag)
    }

    fn get_global_scripts(&self) -> Result<String> {
        let downloaded_scripts = self
            .settings
            .get_site_settings()
            .get_script_urls()
            .par_iter()
            .map(|url| match cache::get_file(url, self.cache.clone()) {
                Ok(script) => script,
                Err(e) => {
                    println!("Failed to download script: {}", e);
                    String::new()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        let script_tag = format!(
            "<script type=\"text/javascript\">{}</script>",
            downloaded_scripts
        );

        Ok(script_tag)
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

            let page_render_data = if let Some(data) = self.settings.get_data_yaml()? {
                PageRenderData {
                    body: body.to_string(),
                    root: site_directory.clone(),
                    data: utils::merge_yaml_values(data, metadata.clone()),
                    remote_data: self.get_remote_data()?,
                }
            } else {
                PageRenderData {
                    body: body.to_string(),
                    data: metadata.clone(),
                    root: site_directory.clone(),
                    remote_data: self.get_remote_data()?,
                }
            };

            let body = self
                .handlebars
                .render_template(&self.get_template(template)?, &page_render_data)?;

            Ok(body)
        } else {
            Ok(body.to_string())
        };

        template
    }

    fn get_remote_data(&self) -> Result<serde_json::Value> {
        match self
            .settings
            .remote_data
            .clone()
            .and_then(|data| data.as_table().cloned())
        {
            Some(data) => {
                let mut result = serde_json::Map::new();

                for (key, url) in data.iter() {
                    match url {
                        toml::Value::String(url) => {
                            let value = cache::get_json(url, self.cache.clone())?;
                            result.insert(key.to_string(), value);
                        }
                        _ => {
                            println!("Failed to get remote data for key: {}", url);
                            result.insert(key.to_string(), serde_json::Value::Null);
                        }
                    }
                }

                Ok(serde_json::Value::Object(result))
            }
            None => Ok(serde_json::Value::Null),
        }
    }
}
