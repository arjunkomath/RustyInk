use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub dev: DevSettings,
    pub site: SiteSettings,
}

#[derive(Debug, Deserialize)]
pub struct DevSettings {
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteSettings {
    pub title: String,
    pub description: String,
    pub top_navigation: Vec<Link>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct PageMetadata {
    pub title: String,
    pub author: String,
    pub date: String,
}
