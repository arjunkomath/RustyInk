use std::{fs, path::Path};

use super::seo;
use crate::shared::{
    logger::Logger,
    settings::{self, Link},
    utils,
};

use super::{cache, handlebar_helpers};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use rayon::prelude::*;
use serde::Serialize;

pub struct Render<'a> {
    file: String,
    theme_dir: String,
    settings: settings::Settings,
    cache: Option<cache::Cache>,
    handlebars: Handlebars<'a>,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
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

    pub fn render_page(
        &self,
        template_name: &str,
        url_path: &str,
        site_directory: &serde_yaml::Value,
    ) -> Result<String> {
        let (metadata, markdown) = self.get_markdown_and_metadata()?;

        let metadata = if let Some(metadata) = metadata {
            let metadata = utils::parse_string_to_yaml(&metadata)?;
            Some(metadata)
        } else {
            None
        };

        // Check if the page is expected to have an amp version
        let is_amp_template = template_name == "amp";
        let is_amp = metadata
            .as_ref()
            .and_then(|metadata| metadata.get("amp"))
            .and_then(|amp| amp.as_bool())
            .unwrap_or(false);
        if is_amp_template && !is_amp {
            return Ok(String::new());
        }

        let content = if let Some(metadata) = &metadata {
            self.render_body(&markdown, metadata, site_directory)
                .with_context(|| format!("Failed to render page: {}", self.file))?
        } else {
            markdown
        };

        let html = self
            .handlebars
            .render_template(
                &self
                    .get_template(template_name)
                    .context("Failed to get template")?,
                &AppRenderData {
                    title: self.settings.meta.title.clone(),
                    description: self.settings.meta.description.clone(),
                    open_graph_tags: seo::generate_open_graph_tags(
                        &self.settings,
                        url_path,
                        is_amp,
                        is_amp_template,
                    )?,
                    content,
                    styles: self.get_global_styles()?,
                    scripts: self.get_global_scripts(),
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
        Ok(Self::split_frontmatter(&markdown).0.map(str::to_string))
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

        let downloaded_styles = self.download_urls(&self.settings.site.style_urls, "style");

        let style_tag = format!("<style>{}{}</style>", downloaded_styles, styles);

        Ok(style_tag)
    }

    fn get_global_scripts(&self) -> String {
        format!(
            "<script type=\"text/javascript\">{}</script>",
            self.download_urls(&self.settings.site.script_urls, "script")
        )
    }

    fn download_urls(&self, urls: &[String], kind: &str) -> String {
        urls.par_iter()
            .map(|url| match cache::get_file(url, self.cache.clone()) {
                Ok(content) => content,
                Err(e) => {
                    Logger::error(&format!("Failed to download {kind}: {e}"));
                    String::new()
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn get_markdown_and_metadata(&self) -> Result<(Option<String>, String)> {
        let markdown = fs::read_to_string(&self.file)?;
        let (metadata, body) = Self::split_frontmatter(&markdown);
        Ok((metadata.map(str::to_string), self.markdown_to_html(body)))
    }

    fn split_frontmatter(markdown: &str) -> (Option<&str>, &str) {
        markdown
            .strip_prefix("---")
            .and_then(|markdown| markdown.split_once("---"))
            .map_or((None, markdown), |(metadata, body)| (Some(metadata), body))
    }

    fn markdown_to_html(&self, markdown: &str) -> String {
        let parser = pulldown_cmark::Parser::new_ext(markdown, pulldown_cmark::Options::all());
        let mut content = String::new();

        if self.settings.site.external_links_open_in_new_tab {
            let events = parser.map(|event| match event {
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link(_, dest, title))
                    if dest.starts_with("http://") || dest.starts_with("https://") =>
                {
                    let mut anchor = String::from("<a href=\"");
                    let _ = pulldown_cmark::escape::escape_href(&mut anchor, &dest);
                    if !title.is_empty() {
                        anchor.push_str("\" title=\"");
                        let _ = pulldown_cmark::escape::escape_html(&mut anchor, &title);
                    }
                    anchor.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
                    pulldown_cmark::Event::Html(anchor.into())
                }
                _ => event,
            });
            pulldown_cmark::html::push_html(&mut content, events);
        } else {
            pulldown_cmark::html::push_html(&mut content, parser);
        }

        content
    }

    fn render_body(
        &self,
        body: &str,
        metadata: &serde_yaml::Value,
        site_directory: &serde_yaml::Value,
    ) -> Result<String> {
        if let Some(template) = metadata.get("template") {
            let template = template
                .as_str()
                .with_context(|| format!("Failed to get template from metadata: {}", self.file))?;

            let data = match &self.settings.data {
                Some(data) => {
                    utils::merge_yaml_values(serde_yaml::to_value(data)?, metadata.clone())
                }
                None => metadata.clone(),
            };
            let page_render_data = PageRenderData {
                body: body.to_string(),
                root: site_directory.clone(),
                data,
                remote_data: self.get_remote_data()?,
            };

            let body = self
                .handlebars
                .render_template(&self.get_template(template)?, &page_render_data)?;

            Ok(body)
        } else {
            Ok(body.to_string())
        }
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
                            Logger::error(&format!("Failed to get remote data for key: {}", url));
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

#[cfg(test)]
mod tests {
    use super::Render;

    #[test]
    fn splits_frontmatter() {
        assert_eq!(
            Render::split_frontmatter("---title: Home---# Hello"),
            (Some("title: Home"), "# Hello")
        );
        assert_eq!(Render::split_frontmatter("# Hello"), (None, "# Hello"));
    }
}
