use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub dev: DevSettings,
    pub site: Option<SiteSettings>,
    pub meta: SiteMeta,
    pub navigation: NavigationSettings,
    pub data: Option<toml::Value>,
    pub remote_data: Option<toml::Value>,
}

impl Settings {
    pub fn default() -> Self {
        Self {
            dev: DevSettings {
                port: 3000,
                ws_port: 3001,
            },
            site: None,
            meta: SiteMeta {
                title: "RustyInk".to_string(),
                description: "A blazing fast static site generator".to_string(),
                og_image_url: None,
                base_url: None,
            },
            navigation: NavigationSettings {
                links: Vec::<Link>::from([Link {
                    label: "Home".to_string(),
                    url: "/".to_string(),
                }]),
            },
            data: None,
            remote_data: None,
        }
    }

    pub fn to_toml_string(&self) -> Result<String> {
        let toml = toml::to_string(self)?;
        Ok(toml)
    }

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

    pub fn get_data_yaml(&self) -> Result<Option<serde_yaml::Value>> {
        if let Some(data) = &self.data {
            let data = serde_yaml::to_value(data)?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevSettings {
    pub port: u16,
    pub ws_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSettings {
    pub block_search_indexing: Option<bool>,
    pub script_urls: Option<Vec<String>>,
    pub style_urls: Option<Vec<String>>,
}

impl SiteSettings {
    pub fn is_search_engine_blocked(&self) -> bool {
        matches!(self.block_search_indexing, Some(true))
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
        self.og_image_url.as_ref().cloned()
    }

    pub fn get_base_url(&self) -> Option<String> {
        self.base_url.as_ref().cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationSettings {
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}
