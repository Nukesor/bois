use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::{info, trace};
use serde::{Deserialize, Serialize};

use super::directory::*;
use crate::{config::file::FileConfig, state::file_parser::read_file, templating::render_template};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum Entry {
    File(File),
    Directory(Directory),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct File {
    /// The relative path to the source file.
    /// Relative to the root directory of the configuration (i.e. Host/Group directory).
    /// We need this information to determine the destination on the target file system.
    pub relative_path: PathBuf,

    /// The parsed configuration block for this file, if one exists.
    #[serde(default)]
    pub config: FileConfig,

    /// The actual configuration file's content, without the bois configuration block.
    pub content: String,

    /// The metadata of the source file.
    /// Used to determine the mode in case it isn't overwritten.
    pub mode: u32,
}

impl File {
    /// By default, the destination path calculates as follows.
    /// Default target directory (based on host/group/default) + relative path of this file from
    /// host/group.
    ///
    /// However, if a path override exists, we always use it.
    /// - If it's an absoulte path, we just use that path. This can be used to deploy files
    ///   **outside** the default target dir.
    /// - If it's a relative path, we just append it to the target_dir.
    pub fn file_path(&self, root: &Path) -> PathBuf {
        let mut path = if let Some(path) = &self.config.path() {
            if path.is_absolute() {
                path.clone()
            } else {
                root.join(path)
            }
        } else {
            root.join(&self.relative_path)
        };

        // If the a rename is requested, set the file name
        if let Some(file_name) = &self.config.rename {
            path.set_file_name(file_name);
        }

        path
    }

    /// Return the mode of the file.
    ///
    /// If an override via `self.config.mode` exists, that mode is used.
    /// Otherwise, we use the mode of the original file.
    pub fn mode(&self) -> u32 {
        self.config.mode.unwrap_or(self.mode)
    }
}

/// Process a directory entry.
/// This function is a convenient wrapper that calls the `read_{directory|file}` functions.
/// In here we do some preparation, such as appending the name of the entry to the relative path
/// and the path_override, if applicable.
///
/// Params:
/// `root` The root of the bois configuration directory.
///        We need this to be able to read the file from the filesystem.
/// `entry` The actual file entry.
/// `directory` The representation of the directory we're currently processing.
///             All files/directories must be added to this `Directory`.
pub fn read_entry(
    root: &Path,
    relative_path: &Path,
    entry: DirEntry,
    directory: &mut Directory,
    mut path_override: Option<PathBuf>,
    template_vars: &serde_yaml::Value,
) -> Result<()> {
    let file_name = entry.file_name();

    let relative_path = relative_path.join(&file_name);

    // If there's an active override, adjust the override for the next level.
    if let Some(path) = path_override {
        path_override = Some(path.join(&file_name));
    }

    // Recursively discover new directories
    let path = entry.path();
    if path.is_dir() {
        let sub_directory = read_directory(root, &relative_path, path_override, template_vars)?;
        directory.entries.push(Entry::Directory(sub_directory));
    } else if path.is_file() {
        trace!("Reading file {path:?}");
        let mut file = read_file(root, &relative_path)?;

        // Check if there's an active path override from a parent directory.
        // If the file doesn't have its own override, use the one from the parent.
        if let Some(path_override) = path_override {
            if file.config.path().is_none() {
                file.config.override_path(path_override);
            }
        }

        // Perform templating, if enabled
        // Otherwise return the raw content.
        if file.config.template {
            info!("Starting templating for file {path:?}");
            file.content = render_template(&file.content, template_vars, &file.config.delimiters)
                .context(format!("Error for template at {path:?}"))?
        };

        directory.entries.push(Entry::File(file));
    }

    Ok(())
}
