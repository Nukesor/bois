use std::fs::{read_to_string, DirEntry};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::debug;
use pest::Parser;
use serde_derive::{Deserialize, Serialize};

use super::directory::*;
use super::parser::{ConfigParser, Rule};
use crate::error::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Entry {
    File(File),
    Directory(Directory),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct File {
    /// The path to the source file.
    /// Relative to the root directory of the configuration.
    pub path: PathBuf,

    /// The parsed configuration block for this file, if one exists.
    #[serde(default)]
    pub config: FileConfig,

    /// The actual configuration file's content, without the bois configuration block.
    pub content: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FileConfig {
    pub owner: Option<String>,
    pub group: Option<String>,
    /// This is represented as a octal `Oo755` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub permissions: Option<u32>,
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

    let entry_relative_path = relative_path.to_path_buf().join(&file_name);

    // Recursively discover new directories
    let path = entry.path();
    if path.is_dir() {
        let sub_directory = read_directory(root, &entry_relative_path)?;
        directory.entries.push(Entry::Directory(sub_directory));
    } else if path.is_file() {
        let mut file_content = read_to_string(&path)
            .map_err(|err| Error::IoPath(path.clone(), "reading file at", err))?;

        let mut file_config = FileConfig::default();

        // Check if there's the key word for a in-file configuration block.
        // If so, try to parse the file as such.
        if file_content.contains("bois_config_start") {
            // Parse the bois configuration block
            let mut parsed = ConfigParser::parse(Rule::full_config, &file_content)
                .context(format!("Failed to parse config block in file at {path:?}"))?;

            // The first parsed block is the bois configuration.
            let config_text = parsed.next().unwrap().to_string();
            // The second parsed block is the actual content of the file.
            file_content = parsed.next().unwrap().to_string();
            debug!("Found config block in file {path:?}:\n{config_text}");

            // Try to deserialize the bois configuration content into the correct struct.
            file_config = serde_yaml::from_str(&config_text).context(format!(
                "Failed to deserialize bois config inside of file {path:?}"
            ))?;
        }

        // Create a new representation of a file with all necessary information.
        let file = File {
            path,
            config: file_config,
            content: file_content,
        };

        // Add it to the current directory.
        directory.entries.push(Entry::File(file));
    }

    Ok(())
}
