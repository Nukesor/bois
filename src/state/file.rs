use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::trace;
use serde_derive::{Deserialize, Serialize};

use super::directory::*;
use crate::constants::{CURRENT_GROUP, CURRENT_USER};
use crate::state::parser::parse_file;

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

/// Read and, if applicable, parse a single configuration file.
/// Add the file to the `&mut Directory`.
///
pub fn read_file(
    root: &Path,
    relative_path: &Path,
    entry: DirEntry,
    directory: &mut Directory,
) -> Result<()> {
    let file_name = entry.file_name().to_string_lossy().to_string();

    // Don't include our own configuration files.
    if file_name == "bois.yml" {
        return Ok(());
    }

    let relative_path = relative_path.to_path_buf().join(&file_name);

    // Recursively discover new directories
    let path = entry.path();
    if path.is_dir() {
        let sub_directory = read_directory(root, &relative_path)?;
        directory.entries.push(Entry::Directory(sub_directory));
    } else if path.is_file() {
        trace!("Reading file {path:?}");
        let file = parse_file(root, &relative_path)?;
        directory.entries.push(Entry::File(file));
    }

    Ok(())
}

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
