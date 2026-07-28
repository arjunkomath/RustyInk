use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub dev: DevSettings,
    #[serde(default)]
    pub site: SiteSettings,
    pub meta: SiteMeta,
    pub navigation: NavigationSettings,
    pub data: Option<toml::Value>,
    pub remote_data: Option<toml::Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dev: DevSettings { port: 3000 },
            site: SiteSettings::default(),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevSettings {
    pub port: u16,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SiteSettings {
    #[serde(default)]
    pub block_search_indexing: bool,
    #[serde(default)]
    pub external_links_open_in_new_tab: bool,
    #[serde(default)]
    pub script_urls: Vec<String>,
    #[serde(default)]
    pub style_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMeta {
    pub title: String,
    pub description: String,
    pub og_image_url: Option<String>,
    pub base_url: Option<String>,
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
