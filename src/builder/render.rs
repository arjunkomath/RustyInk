use std::fs;

use super::settings::{self, Link, PageMetadata, Settings};
use anyhow::Result;
use config::Config;
use handlebars::Handlebars;
use regex::Regex;
use serde::{Deserialize, Serialize};

const CODE_HIGHIGHTING_STYLES: &'static str = r#"
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/styles/atom-one-dark.min.css" integrity="sha512-Jk4AqjWsdSzSWCSuQTfYRIF84Rq/eV0G2+tu07byYwHcbTGfdmLrHjUSwvzp5HvbiqK4ibmNwdcG49Y5RGYPTg==" crossorigin="anonymous" referrerpolicy="no-referrer" />
"#;

const CODE_HIGHIGHTING_SCRIPTS: &'static str = r#"
  <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/highlight.min.js" integrity="sha512-rdhY3cbXURo13l/WU9VlaRyaIYeJ/KBakckXIvJNAQde8DgpOmE+eZf7ha4vdqVjTtwQt69bD2wH2LXob/LB7Q==" crossorigin="anonymous" referrerpolicy="no-referrer"></script>
  <script type="text/javascript">hljs.highlightAll();</script>
"#;

pub struct Render {
    pub file: String,
    pub theme_dir: String,
    pub settings: settings::Settings,
}

#[derive(Serialize, Deserialize)]
struct RenderData {
    meta_title: String,
    meta_description: String,

    links: String,

    content: String,

    styles: String,

    code_highighting_styles: String,
    code_highighting_scripts: String,
}

impl Render {
    pub fn new(file: &str, theme_dir: &str, settings: settings::Settings) -> Self {
        Self {
            file: file.to_string(),
            theme_dir: theme_dir.to_string(),
            settings: settings.clone(),
        }
    }

    pub fn render(&self) -> Result<String> {
        let (metadata, body) = self.get_markdown_and_metadata()?;

        let (template_name, meta_title, meta_description, content) =
            if let Some(metadata) = metadata {
                let metadata: settings::PageMetadata = Config::builder()
                    .add_source(config::File::from_str(&metadata, config::FileFormat::Yaml))
                    .build()
                    .unwrap()
                    .try_deserialize()
                    .unwrap();

                let template_name = metadata.template.clone().unwrap_or("app".to_string());
                let title = format!("{} | {}", self.settings.meta.title, metadata.title);
                let description = self.settings.meta.description.clone();
                let content = self.render_article(&body, Some(metadata));

                (template_name, title, description, content)
            } else {
                let content = self.render_article(&body, None);

                (
                    "app".to_string(),
                    self.settings.meta.title.clone(),
                    self.settings.meta.description.clone(),
                    content,
                )
            };

        let links = self.render_links(&self.settings.navigation.links);

        let styles = self.get_global_styles()?;

        let (code_highighting_styles, code_highighting_scripts) =
            self.handle_code_highlighting(&self.settings)?;

        let html = Handlebars::new().render_template(
            &self.get_template(&template_name)?,
            &RenderData {
                meta_title,
                meta_description,
                links,
                content,
                styles,
                code_highighting_styles,
                code_highighting_scripts,
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

    fn handle_code_highlighting(&self, settings: &Settings) -> Result<(String, String)> {
        let enabled = match settings.site.as_ref() {
            Some(site) => site.code_highlighting.unwrap_or(false),
            None => false,
        };

        if enabled {
            Ok((
                String::from(CODE_HIGHIGHTING_STYLES),
                String::from(CODE_HIGHIGHTING_SCRIPTS),
            ))
        } else {
            Ok((String::new(), String::new()))
        }
    }

    pub fn render_links(&self, links: &Vec<Link>) -> String {
        let mut nav_links = String::new();

        for link in links {
            nav_links.push_str(
                format!(r#"<li><a href="{}">{}</a></li>"#, link.url, link.label).as_str(),
            );
        }

        nav_links
    }

    pub fn render_article(&self, body: &str, metadata: Option<PageMetadata>) -> String {
        if let Some(metadata) = metadata {
            let mut header_extras: Vec<String> = vec![];

            let author = if let Some(author) = &metadata.author {
                format!("// Written by {}", author)
            } else {
                String::new()
            };
            let author = if let Some(author_url) = &metadata.author_url {
                format!(
                    "<a target=\"_blank\" rel=\"noopener noreferrer\" href=\"{}\">{}</a>",
                    author_url, author
                )
            } else {
                author
            };
            header_extras.push(format!("<p><small>{}</small></p>", author));

            let published = if let Some(published) = &metadata.published {
                format!("<time datetime=\"{}\">// {}</time>", published, published)
            } else {
                String::new()
            };
            header_extras.push(format!("<p><small>{}</small></p>", published));

            format!(
                r#"<article>
            <header>
              <h2>{}</h2>
              {}
            </header>
    {}
            <footer><small>{}</small></footer>
    </article>
    "#,
                metadata.title,
                header_extras.join(""),
                body,
                metadata.footnote.unwrap_or(String::new())
            )
        } else {
            format!(
                r#"<article>
    {}
    </article>"#,
                body
            )
        }
    }
}
