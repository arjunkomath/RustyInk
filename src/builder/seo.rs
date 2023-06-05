use super::settings::Settings;
use anyhow::Result;

pub fn generate_robots_txt(settings: &Settings) -> Result<String> {
    let blocked = match settings.site.as_ref() {
        Some(site) => site.block_search_indexing.unwrap_or(false),
        None => false,
    };

    let robots = if blocked {
        String::from("User-agent: *\nDisallow: /")
    } else {
        String::from("User-agent: *\nAllow: /")
    };

    Ok(robots)
}
