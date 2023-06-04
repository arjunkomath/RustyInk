use std::{fs, path::PathBuf};

use anyhow::Result;

pub fn create_dir_in_path(path: &PathBuf) -> Result<()> {
    fs::create_dir(&path)?;

    Ok(())
}

pub fn path_to_string(path: &PathBuf) -> String {
    path.canonicalize().unwrap().to_str().unwrap().to_string()
}
