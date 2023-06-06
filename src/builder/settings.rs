use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub dev: DevSettings,
    pub site: Option<SiteSettings>,
    pub meta: SiteMeta,
    pub navigation: NavigationSettings,
}

impl Settings {
    pub fn new() -> String {
        let default_settings = String::from(
            r#"[dev]
port = 3000

[meta]
title = "Rusty!nk"
description = "Blazing fast static site generator written in Rust"

[navigation]
links = [
    { label = "~/", url = "/" },
    { label = "GitHub", url = "https://github.com/arjunkomath/rustyink" },
]"#,
        );

        default_settings
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DevSettings {
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteSettings {
    pub block_search_indexing: Option<bool>,
    pub sitemap_base_url: Option<String>,
    pub code_highlighting: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMeta {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NavigationSettings {
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageMetadata {
    pub title: String,
    pub author: String,
}
