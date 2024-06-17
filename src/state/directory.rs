use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use log::trace;
use serde_derive::{Deserialize, Serialize};

use super::file::*;
use crate::constants::CURRENT_GROUP;
use crate::constants::CURRENT_USER;
use crate::{error::Error, helper::read_yaml};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Directory {
    /// The relative path to the source file.
    /// Relative to the root directory of the configuration (i.e. Host/Group directory).
    /// We need this information to determine the destination on the target file system.
    pub relative_path: PathBuf,
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub config: DirectoryConfig,
}

impl Directory {
    pub fn new(path: &Path) -> Directory {
        Directory {
            relative_path: path.to_path_buf(),
            ..Directory::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryConfig {
    /// If this is set, this path will be used as a destination.
    /// If it's an relative path, it'll be treated as relative to the default target directory.
    /// If it's an absolute path, that absolute path will be used.
    pub path: Option<PathBuf>,
    pub owner: Option<String>,
    pub group: Option<String>,
    /// This is represented as a octal `Oo755` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub permissions: Option<u32>,
}

/// Recursively discover all bois and non-bois configuration files in a group directory.
/// Params:
/// `root` The root of the bois configuration directory.
///     We need this to be able to read the file from the filesystem.
/// `relative_path` The path of the actual directory relative to the root of the
///     bois configuration directory. `root + relative_path => actual path`
///     This is used to determine the destination path, relative to the target directory.
/// `path_override`
pub fn read_directory(
    root: &Path,
    relative_path: &Path,
    mut path_override: Option<PathBuf>,
) -> Result<Directory> {
    let directory_path = root.join(relative_path);
    trace!("Entered directory {directory_path:?}");

    // Read the `bois.yml` from the directory if it exists, otherwise fall back to default.
    let mut directory_config = DirectoryConfig::default();
    if directory_path.join("bois.yml").exists() || directory_path.join("bois.yaml").exists() {
        directory_config = read_yaml::<DirectoryConfig>(&directory_path, "bois")?;
    }

    // Check if there's a new path override in this config.
    // If it is, we set the override, which will be passed to all child entries.
    if let Some(path) = &directory_config.path {
        path_override = Some(path.clone());
    } else if let Some(path) = &path_override {
        directory_config.path = Some(path.clone());
    }

    let entries = std::fs::read_dir(&directory_path)
        .map_err(|err| Error::IoPath(directory_path.clone(), "reading directory", err))?;

    // Create the representation for this directory
    let mut directory = Directory {
        relative_path: relative_path.to_path_buf(),
        entries: Vec::new(),
        config: directory_config,
    };

    // Go through all entries in this directory
    for entry in entries {
        let entry =
            entry.map_err(|err| Error::IoPath(directory_path.clone(), "reading entry", err))?;

        read_entry(
            root,
            relative_path,
            entry,
            &mut directory,
            path_override.clone(),
        )?;
    }

    Ok(directory)
}

/// This impl block contains convenience getters for directory metadata, which fall back to
/// default values.
impl DirectoryConfig {
    pub fn permissions(&self) -> u32 {
        self.permissions.unwrap_or(0o755)
    }

    pub fn owner(&self) -> String {
        self.owner.clone().unwrap_or(CURRENT_USER.clone())
    }

    pub fn group(&self) -> String {
        self.group.clone().unwrap_or(CURRENT_GROUP.clone())
    }
}
