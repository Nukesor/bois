use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};

use super::file::*;
use crate::error::Error;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Directory {
    pub path: PathBuf,
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub config: DirectoryConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DirectoryConfig {
    pub owner: Option<String>,
    pub group: Option<String>,
    /// This is represented as a octal `Oo755` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub permissions: Option<u32>,
}

/// Recursively discover all bois and non-bois configuration files in a group directory.
pub fn read_directory(root: &PathBuf, relative_path: &PathBuf) -> Result<Directory> {
    let directory_path = root.join(relative_path);
    let entries = std::fs::read_dir(&directory_path)
        .map_err(|err| Error::IoPathError(directory_path.clone(), "reading", err))?;

    let mut directory_config = DirectoryConfig::default();
    // Check if there's a directory configuration file in here.
    let dir_config_path = directory_path.join("bois.yml");
    if dir_config_path.exists() {
        let content = read_to_string(&dir_config_path)
            .map_err(|err| Error::IoPathError(dir_config_path.clone(), "reading", err))?;

        let config: DirectoryConfig = serde_yaml::from_str(&content).context(format!(
            "Failed serializing config at path: {dir_config_path:?}"
        ))?;
        directory_config = config;
    }

    // Create the representation for this directory
    let mut directory = Directory {
        path: relative_path.clone(),
        entries: Vec::new(),
        config: directory_config,
    };

    // Go through all entries in this directory
    for entry in entries {
        let entry = entry
            .map_err(|err| Error::IoPathError(directory_path.clone(), "reading entry", err))?;

        read_file(root, relative_path, entry, &mut directory)?;
    }

    Ok(directory)
}
