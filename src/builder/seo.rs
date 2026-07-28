use anyhow::{Context, Result};
use chrono::prelude::*;
use sitewriter::{ChangeFreq, UrlEntry};

use crate::shared::settings::Settings;

pub fn generate_robots_txt(settings: &Settings) -> Result<String> {
    let robots = if settings.site.block_search_indexing {
        String::from("User-agent: *\nDisallow: /")
    } else {
        let sitemap_base_url = settings
            .meta
            .base_url
            .as_deref()
            .context("No sitemap base url found in Settings.toml")?;

        format!("User-agent: *\nAllow: /\nSitemap: {sitemap_base_url}/sitemap.xml")
    };

    Ok(robots)
}

pub fn generate_sitemap_xml(
    settings: &Settings,
    all_url_paths: &Vec<(String, String)>,
) -> Result<String> {
    let sitemap_base_url = settings
        .meta
        .base_url
        .as_deref()
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
    let title = &settings.meta.title;
    let description = &settings.meta.description;
    let base_url = settings
        .meta
        .base_url
        .as_deref()
        .context("No base url found in Settings.toml")?;

    let mut tags = format!(
        "<meta property=\"title\" content=\"{title}\" />\n\
         <meta name=\"description\" content=\"{description}\" />\n\
         <meta property=\"og:type\" content=\"website\" />\n\
         <meta property=\"og:title\" content=\"{title}\" />\n\
         <meta property=\"og:description\" content=\"{description}\" />\n\
         <meta name=\"twitter:title\" content=\"{title}\" />\n\
         <meta name=\"twitter:description\" content=\"{description}\" />\n\
         <meta property=\"og:url\" content=\"{base_url}\" />\n\
         <meta name=\"twitter:url\" content=\"{base_url}\" />"
    );

    if is_amp && !is_amp_template {
        tags.push_str(&format!(
            "\n<link rel=\"amphtml\" href=\"{base_url}{url_path}amp\">"
        ));
    }

    tags.push_str(&format!(
        "\n<link rel=\"canonical\" href=\"{base_url}{url_path}\">"
    ));

    if let Some(og_image_url) = &settings.meta.og_image_url {
        tags.push_str(&format!(
            "\n<meta property=\"og:image\" content=\"{og_image_url}\" />\n\
             <meta name=\"twitter:image\" content=\"{og_image_url}\" />\n\
             <meta name=\"twitter:card\" content=\"summary_large_image\" />"
        ));
    }

    Ok(tags)
}
