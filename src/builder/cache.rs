use std::{fs, path::Path};

use anyhow::Result;
use md5::{Digest, Md5};

#[derive(Debug, Clone)]
pub struct Cache {
    cache_dir: String,
}

impl Cache {
    pub fn new(cache_dir: String) -> Result<Self> {
        Ok(Cache { cache_dir })
    }

    pub fn get_cache_file_path(&self, key: &str) -> String {
        let mut hasher = Md5::new();
        hasher.update(key);
        let key = hasher.finalize();
        let key = format!("{:x}", key);
        format!("{}/{}.txt", self.cache_dir, key)
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
        if let Ok(contents) = fs::read_to_string(file_path) {
            Some(contents)
        } else {
            None
        }
    }

    pub fn clean(&self) -> Result<()> {
        fs::remove_dir_all(&self.cache_dir)?;
        Ok(())
    }
}
