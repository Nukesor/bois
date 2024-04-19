use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use super::file::*;
use crate::{error::Error, helper::read_yaml};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Directory {
    pub path: PathBuf,
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub config: DirectoryConfig,
}

impl Directory {
    pub fn new(path: &Path) -> Directory {
        Directory {
            path: path.to_path_buf(),
            ..Directory::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryConfig {
    pub owner: Option<String>,
    pub group: Option<String>,
    /// This is represented as a octal `Oo755` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub permissions: Option<u32>,
}

/// Recursively discover all bois and non-bois configuration files in a group directory.
pub fn read_directory(root: &Path, relative_path: &Path) -> Result<Directory> {
    let directory_path = root.join(relative_path);
    // Read the `host.yml` from the host directory.
    let directory_config = read_yaml::<DirectoryConfig>(&directory_path, "bois")?;

    let entries = std::fs::read_dir(&directory_path)
        .map_err(|err| Error::IoPathError(directory_path.clone(), "reading", err))?;

    // Create the representation for this directory
    let mut directory = Directory {
        path: relative_path.to_path_buf(),
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
