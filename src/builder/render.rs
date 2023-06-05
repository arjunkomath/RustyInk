use std::fs;

use super::{
    base,
    settings::{self, Settings},
};
use anyhow::Result;
use config::Config;
use handlebars::Handlebars;
use regex::Regex;

pub struct Render {
    pub file: String,
    pub styles_file: String,
    pub settings: settings::Settings,
}

impl Render {
    pub fn new(file: &str, styles_file: &str, settings: settings::Settings) -> Self {
        Self {
            file: file.to_string(),
            styles_file: styles_file.to_string(),
            settings: settings.clone(),
        }
    }

    pub fn render(&self) -> Result<String> {
        let mut html = base::HEADER.to_owned();

        let (metadata, body) = self.get_markdown_and_metadata()?;

        if let Some(metadata) = metadata {
            let metadata: settings::PageMetadata = Config::builder()
                .add_source(config::File::from_str(&metadata, config::FileFormat::Yaml))
                .build()
                .unwrap()
                .try_deserialize()
                .unwrap();

            html.push_str(&base::render_article(&body, Some(metadata)).as_str());
        } else {
            html.push_str(&base::render_article(&body, None).as_str());
        }

        html.push_str(base::FOOTER);

        let reg = Handlebars::new();
        let html = reg.render_template(&html, &self.settings.meta)?;

        let top_navigation = base::render_links(&self.settings.navigation.links);
        let html = html.replace("%%LINKS%%", &top_navigation);

        let global_styles = self.get_global_styles()?;
        let html = html.replace("%%STYLES%%", &global_styles);

        let html = match self.handle_code_highlighting(&html, &self.settings) {
            Ok(html) => html,
            _ => html,
        };

        Ok(html)
    }

    fn get_global_styles(&self) -> Result<String> {
        let styles = fs::read_to_string(&self.styles_file)?;

        Ok(styles)
    }

    fn get_markdown_and_metadata(&self) -> Result<(Option<String>, String)> {
        let markdown = fs::read_to_string(&self.file)?;

        let metadata = Regex::new(r"(?s)---(.*?)---(.*)").unwrap();
        if let Some(captures) = metadata.captures(&markdown) {
            let metadata = captures.get(1).unwrap().as_str();
            let markdown = captures.get(2).unwrap().as_str();

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

    fn handle_code_highlighting(&self, html: &str, settings: &Settings) -> Result<String> {
        let enabled = match settings.site.as_ref() {
            Some(site) => site.code_highlighting.unwrap_or(false),
            None => false,
        };

        let result = if enabled {
            html.replace("%%CODE_HIGHIGHTING_STYLES%%", base::CODE_HIGHIGHTING_STYLES)
                .replace(
                    "%%CODE_HIGHIGHTING_SCRIPTS%%",
                    base::CODE_HIGHIGHTING_SCRIPTS,
                )
        } else {
            html.replace("%%CODE_HIGHIGHTING_STYLES%%", "")
                .replace("%%CODE_HIGHIGHTING_SCRIPTS%%", "")
        };

        Ok(result)
    }
}
