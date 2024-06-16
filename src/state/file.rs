use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::trace;
use serde_derive::{Deserialize, Serialize};

use super::directory::*;
use crate::constants::{CURRENT_GROUP, CURRENT_USER};
use crate::state::parser::read_file;

#[derive(Clone, Debug, Deserialize, Serialize)]
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
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FileConfig {
    /// If this is set, this path will be used as a destination.
    /// If it's an relative path, it'll be treated as relative to the default target directory.
    /// If it's an absolute path, that absolute path will be used.
    pub path: Option<PathBuf>,
    pub owner: Option<String>,
    pub group: Option<String>,
    /// This is represented as a octal `Oo640` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub permissions: Option<u32>,
    /// Whether this file should be treated as a template.
    /// Defaults to `false` to prevent unwanted behavior.
    #[serde(default)]
    pub template: bool,
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
) -> Result<()> {
    let file_name = entry.file_name();

    // Don't include our own configuration files.
    // Those are already handled in the `read_directory` function.
    if file_name == "bois.yml" {
        return Ok(());
    }

    let relative_path = relative_path.join(&file_name);

    // If there's an active override, adjust the override for the next level.
    if let Some(path) = path_override {
        path_override = Some(path.join(&file_name));
    }

    // Recursively discover new directories
    let path = entry.path();
    if path.is_dir() {
        let sub_directory = read_directory(root, &relative_path, path_override)?;
        directory.entries.push(Entry::Directory(sub_directory));
    } else if path.is_file() {
        trace!("Reading file {path:?}");
        let file = read_file(root, &relative_path, path_override)?;
        directory.entries.push(Entry::File(file));
    }

    Ok(())
}

/// This impl block contains convenience getters for file metadata, which fall back to
/// default values.
impl FileConfig {
    pub fn permissions(&self) -> u32 {
        self.permissions.unwrap_or(0o640)
    }

    pub fn owner(&self) -> String {
        self.owner.clone().unwrap_or(CURRENT_USER.clone())
    }

    pub fn group(&self) -> String {
        self.group.clone().unwrap_or(CURRENT_GROUP.clone())
    }
}
