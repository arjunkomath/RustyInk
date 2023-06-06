use super::settings::Settings;
use anyhow::Result;
use chrono::prelude::*;
use sitewriter::{ChangeFreq, UrlEntry};

pub fn generate_robots_txt(settings: &Settings) -> Result<String> {
    let blocked = match settings.site.as_ref() {
        Some(site) => site.block_search_indexing.unwrap_or(false),
        None => false,
    };

    let sitemap_base_url = match settings.site.as_ref() {
        Some(site) => site.sitemap_base_url.clone().unwrap_or(String::new()),
        None => String::new(),
    };

    let robots = if blocked {
        String::from("User-agent: *\nDisallow: /")
    } else {
        String::from("User-agent: *\nAllow: /\nSitemap: ")
            + &sitemap_base_url
            + &String::from("/sitemap.xml")
    };

    Ok(robots)
}

pub fn generate_sitemap_xml(
    settings: &Settings,
    output_dir: &str,
    all_file_paths: &Vec<String>,
) -> Result<Option<String>> {
    let sitemap_base_url = match settings.site.as_ref() {
        Some(site) => site.sitemap_base_url.clone().unwrap_or(String::new()),
        None => String::new(),
    };

    if sitemap_base_url.is_empty() {
        return Ok(None);
    }

    let mut urls = vec![];

    for file in all_file_paths {
        let canonical_url = file
            .replace(&output_dir, &sitemap_base_url)
            .replace("index.html", "");

        urls.push(UrlEntry {
            loc: canonical_url.parse().unwrap(),
            changefreq: Some(ChangeFreq::Weekly),
            priority: Some(0.8),
            lastmod: Some(Utc::now()),
        });
    }

    let xml = sitewriter::generate_str(&urls);

    Ok(Some(xml))
}
