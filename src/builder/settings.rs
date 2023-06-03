use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub dev: DevSettings,
    pub site: SiteSettings,
    pub navigation: NavigationSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DevSettings {
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSettings {
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
