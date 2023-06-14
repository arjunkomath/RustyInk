use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub dev: DevSettings,
    pub site: Option<SiteSettings>,
    pub meta: SiteMeta,
    pub navigation: NavigationSettings,
}

impl Settings {
    pub fn get_site_settings(&self) -> SiteSettings {
        match &self.site {
            Some(site) => site.clone(),
            None => SiteSettings {
                block_search_indexing: Some(false),
                script_urls: Some(Vec::<String>::new()),
                style_urls: Some(Vec::<String>::new()),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DevSettings {
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteSettings {
    pub block_search_indexing: Option<bool>,
    pub script_urls: Option<Vec<String>>,
    pub style_urls: Option<Vec<String>>,
}

impl SiteSettings {
    pub fn is_search_engine_blocked(&self) -> bool {
        match self.block_search_indexing {
            Some(true) => true,
            _ => false,
        }
    }

    pub fn get_script_urls(&self) -> Vec<String> {
        match &self.script_urls {
            Some(urls) => urls.clone(),
            None => Vec::<String>::new(),
        }
    }

    pub fn get_style_urls(&self) -> Vec<String> {
        match &self.style_urls {
            Some(urls) => urls.clone(),
            None => Vec::<String>::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMeta {
    pub title: String,
    pub description: String,
    pub og_image_url: Option<String>,
    pub base_url: Option<String>,
}

impl SiteMeta {
    pub fn get_og_image_url(&self) -> Option<String> {
        match &self.og_image_url {
            Some(url) => Some(url.clone()),
            None => None,
        }
    }

    pub fn get_base_url(&self) -> Option<String> {
        match &self.base_url {
            Some(url) => Some(url.clone()),
            None => None,
        }
    }
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
