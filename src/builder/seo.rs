use anyhow::{Context, Result};
use chrono::prelude::*;
use sitewriter::{ChangeFreq, UrlEntry};

use crate::shared::settings::Settings;

pub fn generate_robots_txt(settings: &Settings) -> Result<String> {
    let blocked = settings.get_site_settings().is_search_engine_blocked();

    let robots = if blocked {
        String::from("User-agent: *\nDisallow: /")
    } else {
        let sitemap_base_url = settings
            .meta
            .get_base_url()
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
        .meta
        .get_base_url()
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

pub fn generate_open_graph_tags(
    settings: &Settings,
    url_path: &str,
    is_amp: bool,
    is_amp_template: bool,
) -> Result<String> {
    let title = settings.meta.title.clone();
    let description = settings.meta.description.clone();
    let base_url = settings
        .meta
        .get_base_url()
        .context("No base url found in Settings.toml")?;

    let mut tags = vec![];

    // Primary meta tags
    tags.push(format!("<meta property=\"title\" content=\"{}\" />", title));
    tags.push(format!(
        "<meta name=\"description\" content=\"{}\" />",
        description
    ));
    tags.push(format!(
        "<meta property=\"og:title\" content=\"{}\" />",
        title
    ));
    tags.push(format!(
        "<meta property=\"og:description\" content=\"{}\" />",
        description
    ));

    if is_amp && !is_amp_template {
        tags.push(format!(
            "<link rel=\"amphtml\" href=\"{}{}amp\">",
            base_url, url_path
        ));
    } else if is_amp_template && is_amp {
        tags.push(format!(
            "<link rel=\"canonical\" href=\"{}{}\">",
            base_url, url_path
        ));
    }

    // Open Graph / Facebook
    tags.push(String::from(
        "<meta property=\"og:type\" content=\"website\" />",
    ));
    tags.push(format!(
        "<meta property=\"og:title\" content=\"{}\" />",
        title
    ));
    tags.push(format!(
        "<meta property=\"og:description\" content=\"{}\" />",
        description
    ));

    // Twitter
    tags.push(format!(
        "<meta name=\"twitter:title\" content=\"{}\" />",
        title
    ));
    tags.push(format!(
        "<meta name=\"twitter:description\" content=\"{}\" />",
        description
    ));

    if let Some(base_url) = settings.meta.get_base_url() {
        tags.push(format!(
            "<meta property=\"og:url\" content=\"{}\" />",
            base_url
        ));
        tags.push(format!(
            "<meta name=\"twitter:url\" content=\"{}\" />",
            base_url
        ));
    }

    if let Some(og_image_url) = settings.meta.get_og_image_url() {
        tags.push(format!(
            "<meta property=\"og:image\" content=\"{}\" />",
            og_image_url
        ));
        tags.push(format!(
            "<meta name=\"twitter:image\" content=\"{}\" />",
            og_image_url
        ));
        tags.push(String::from(
            "<meta name=\"twitter:card\" content=\"summary_large_image\" />",
        ));
    }

    Ok(tags.join("\n"))
}
