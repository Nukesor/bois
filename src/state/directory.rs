use std::path::{Path, PathBuf};

use anyhow::Result;
use log::trace;
use serde::{Deserialize, Serialize};

use super::file::*;
use crate::{
    config::{directory::DirectoryConfig, helper::read_yaml},
    error::Error,
};

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

    /// By default, the destination path calculates as follows.
    /// Default target directory (based on host/group/default) + relative path of this dir from
    /// host/group.
    ///
    /// However, if a path override exists, we always use it.
    /// - If it's an absolute path, we just use that path. This can be used to deploy files
    ///   **outside** the default target dir.
    /// - If it's a relative path, we just append it to the target_dir.
    pub fn file_path(&self, root: &Path) -> PathBuf {
        if let Some(path) = &self.config.path() {
            if path.is_absolute() {
                path.clone()
            } else {
                root.join(path)
            }
        } else {
            root.join(&self.relative_path)
        }
    }
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
    template_vars: &serde_yaml::Value,
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
    if let Some(path) = directory_config.path() {
        path_override = Some(path);
    } else if let Some(path) = &path_override {
        directory_config.override_path(path.clone());
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

        // Don't include our own configuration files.
        // Those are already handled in the `read_directory` function.
        let file_name = entry.file_name();
        if file_name == "bois.yml" {
            continue;
        }

        read_entry(
            root,
            relative_path,
            entry,
            &mut directory,
            path_override.clone(),
            template_vars,
        )?;
    }

    Ok(directory)
}
