use super::settings::Settings;
use anyhow::{Context, Result};
use chrono::prelude::*;
use sitewriter::{ChangeFreq, UrlEntry};

pub fn generate_robots_txt(settings: &Settings) -> Result<String> {
    let blocked = settings.get_site_settings().is_search_engine_blocked();

    let robots = if blocked {
        String::from("User-agent: *\nDisallow: /")
    } else {
        let sitemap_base_url = settings
            .get_site_settings()
            .get_sitemap_base_url()
            .context("No sitemap base url found in Settings.toml")?;

        String::from("User-agent: *\nAllow: /\nSitemap: ")
            + &sitemap_base_url
            + &String::from("/sitemap.xml")
    };

    Ok(robots)
}

pub fn generate_sitemap_xml(
    settings: &Settings,
    all_url_paths: &Vec<(String, String)>,
) -> Result<String> {
    let sitemap_base_url = settings
        .get_site_settings()
        .get_sitemap_base_url()
        .context("No sitemap base url found in Settings.toml")?;

    if sitemap_base_url.is_empty() {
        return Err(anyhow::anyhow!(
            "No sitemap base url found in Settings.toml"
        ));
    }

    let mut urls = vec![];

    for (file, _) in all_url_paths {
        let canonical_url = format!("{}{}", sitemap_base_url, file);

        if let Ok(canonical_url) = canonical_url.parse() {
            urls.push(UrlEntry {
                loc: canonical_url,
                changefreq: Some(ChangeFreq::Weekly),
                priority: Some(0.8),
                lastmod: Some(Utc::now()),
            });
        }
    }

    let xml = sitewriter::generate_str(&urls);

    Ok(xml)
}

pub fn generate_open_graph_tags(settings: &Settings) -> Result<String> {
    let title = settings.meta.title.clone();
    let description = settings.meta.description.clone();

    let mut tags = vec![];

    tags.push(format!(
        "<meta property=\"og:title\" content=\"{}\" />",
        title
    ));
    tags.push(format!(
        "<meta property=\"og:description\" content=\"{}\" />",
        description
    ));

    Ok(tags.join("\n"))
}
