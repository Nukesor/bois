use std::fs::{self, read_to_string};
use std::path::PathBuf;

use anyhow::{Context, Result};

use pest::Parser;
use serde_derive::{Deserialize, Serialize};

use crate::error::Error;
use crate::parser;

/// Paths are usually relative to a given root directory, but it's also possible to specify
/// absolute paths that aren't necessarily in the specified root directory.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Path {
    Relative(PathBuf),
    Absolute(PathBuf),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Entry {
    File(File),
    Directory(Directory),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pre_hooks: Vec<String>,
    post_hooks: Vec<String>,
    permissions: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct File {
    /// The path to the source file.
    /// Relative to the root directory of the configuration.
    path: PathBuf,

    /// The parsed configuration block for this file, if one exists.
    config: Option<FileConfig>,

    /// The configuration file's content, without the bois configuration block.
    content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Directory {
    path: PathBuf,
    entries: Vec<Entry>,
}

pub fn discover_files(root: &PathBuf, relative_path: &PathBuf) -> Result<Directory> {
    let directory_path = root.join(relative_path);
    let entries = fs::read_dir(&directory_path)
        .map_err(|err| Error::IoPathError(directory_path.clone(), "reading", err))?;

    // Create the representation for this directory
    let mut directory = Directory {
        path: relative_path.clone(),
        entries: Vec::new(),
    };

    // Go through all entries in this directory
    for entry in entries {
        let entry = entry
            .map_err(|err| Error::IoPathError(directory_path.clone(), "reading entry", err))?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let entry_relative_path = relative_path.clone().join(&file_name);

        // Recursively discover new directories
        let path = entry.path();
        if path.is_dir() {
            let sub_directory = discover_files(&root, &entry_relative_path)?;
            directory.entries.push(Entry::Directory(sub_directory));
        } else if path.is_file() {
            let file_content = read_to_string(&path)
                .map_err(|err| Error::IoPathError(path.clone(), "reading file at", err))?;

            //let file_config = None;
            // We found a start keyword for a bois configuration block.
            if file_content.contains("bois_config_start") {
                // Check if there's a config
                let parsed = parser::ConfigParser::parse(parser::Rule::full_config, &file_content)
                    .context(format!("Failed to parse config block in file at {path:?}"))?;

                dbg!(parsed);
            }
        }
    }

    Ok(directory)
}
