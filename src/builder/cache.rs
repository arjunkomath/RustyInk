use std::{fs, path::Path};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Cache {
    cache_dir: String,
}

impl Cache {
    pub fn new(cache_dir: String) -> Self {
        Cache { cache_dir }
    }

    pub fn get_cache_file_path(&self, key: &str) -> String {
        let mut hash = 0xcbf29ce484222325_u64;
        for byte in key.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{}/{hash:016x}.txt", self.cache_dir)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let file_path = self.get_cache_file_path(key);

        // Create the directory if it doesn't exist
        if let Some(parent_dir) = Path::new(&file_path).parent() {
            fs::create_dir_all(parent_dir)?;
        }

        fs::write(file_path, value)?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let file_path = self.get_cache_file_path(key);
        fs::read_to_string(file_path).ok()
    }

    pub fn clean(&self) -> Result<()> {
        fs::remove_dir_all(&self.cache_dir)?;
        Ok(())
    }
}

pub fn get_file(url: &str, cache: Option<Cache>) -> Result<String> {
    let content = cache.as_ref().and_then(|c| c.get(url));
    if let Some(content) = content {
        return Ok(content);
    }

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?;
    let content = response.text()?;

    if let Some(cache) = &cache {
        cache.set(url, content.as_str())?;
    }

    Ok(content)
}

pub fn get_json(url: &str, cache: Option<Cache>) -> Result<serde_json::Value> {
    let content = cache.as_ref().and_then(|c| c.get(url));
    if let Some(content) = content {
        return Ok(serde_json::from_str(&content)?);
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("Rusty-Ink")
        .build()?;
    let response = client.get(url).send()?;
    let content = response.text()?;

    if let Some(cache) = &cache {
        cache.set(url, content.as_str())?;
    }

    Ok(serde_json::from_str(&content)?)
}

#[cfg(test)]
mod tests {
    use super::Cache;

    #[test]
    fn cache_paths_are_stable() {
        assert_eq!(
            Cache::new("cache".to_string()).get_cache_file_path("hello"),
            "cache/a430d84680aabd0b.txt"
        );
    }
}
