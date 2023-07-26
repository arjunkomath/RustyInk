use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde_yaml::Value;

pub fn create_dir_in_path(path: &PathBuf) -> Result<()> {
    if fs::metadata(path).is_err() {
        fs::create_dir(path)?;
    }

    Ok(())
}

pub fn path_to_string(path: &PathBuf) -> Result<String> {
    path.canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", path))
        .and_then(|x| {
            x.to_str()
                .context("Failed to parse patht to string")
                .map(|s| s.to_string())
        })
}

pub fn parse_string_to_yaml(string: &str) -> Result<serde_yaml::Value> {
    let metadata: serde_yaml::Value = serde_yaml::from_str(string)?;
    Ok(metadata)
}

pub fn merge_yaml_values(base: Value, other: Value) -> Value {
    match (base, other) {
        (Value::Mapping(mut base_map), Value::Mapping(other_map)) => {
            for (key, other_value) in other_map {
                let base_value = base_map.entry(key).or_insert(Value::Null);
                *base_value = merge_yaml_values(base_value.clone(), other_value);
            }
            Value::Mapping(base_map)
        }
        (_, other) => other,
    }
}
