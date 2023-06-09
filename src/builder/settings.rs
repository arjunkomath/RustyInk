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
                sitemap_base_url: None,
                code_highlighting: Some(false),
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
    pub sitemap_base_url: Option<String>,
    pub code_highlighting: Option<bool>,
}

impl SiteSettings {
    pub fn is_search_engine_blocked(&self) -> bool {
        match self.block_search_indexing {
            Some(true) => true,
            _ => false,
        }
    }

    pub fn get_sitemap_base_url(&self) -> Option<String> {
        match &self.sitemap_base_url {
            Some(url) => Some(url.clone()),
            None => None,
        }
    }

    pub fn is_code_highlighting_enabled(&self) -> bool {
        match self.code_highlighting {
            Some(true) => true,
            _ => false,
        }
    }
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
