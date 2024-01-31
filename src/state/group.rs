use std::fs::{read_to_string, DirEntry};
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use log::debug;
use pest::Parser;
use serde_derive::{Deserialize, Serialize};

use super::file::*;
use super::parser::{ConfigParser, Rule};
use crate::error::Error;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Group {
    /// The top-level configuration file for this group/host.
    pub group_config: GroupConfig,
    /// The content of this group's directory.
    pub directory: Directory,
    /// All variables that're available to all groups during templating.
    pub global_variables: HashMap<String, String>,
    /// All variables that're available during templating for this group.
    pub variables: HashMap<String, String>,
    /// All other groups that should be loaded.
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GroupConfig {
    /// Other groups that're required by this group.
    pub dependencies: String,
}

pub fn read_group(root: &PathBuf, name: &str) -> Result<Group> {
    let directory = read_directory(&root.join(name), &PathBuf::new())?;

    Ok(Group {
        directory,
        ..Default::default()
    })
}

/// Recursively discover all bois and non-bois configuration files in a group directory.
pub fn read_directory(root: &PathBuf, relative_path: &PathBuf) -> Result<Directory> {
    let directory_path = root.join(relative_path);
    let entries = std::fs::read_dir(&directory_path)
        .map_err(|err| Error::IoPathError(directory_path.clone(), "reading", err))?;

    let mut directory_config = None;
    // Check if there's a directory configuration file in here.
    let dir_config_path = directory_path.join("bois.yml");
    if dir_config_path.exists() {
        let content = read_to_string(&dir_config_path)
            .map_err(|err| Error::IoPathError(dir_config_path.clone(), "reading", err))?;

        let config: DirectoryConfig = serde_yaml::from_str(&content).context(format!(
            "Failed serializing config at path: {dir_config_path:?}"
        ))?;
        directory_config = Some(config);
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

/// Read and, if applicable, parse a single configuration file.
/// Add the file to the `&mut Directory`.
fn read_file(
    root: &PathBuf,
    relative_path: &PathBuf,
    entry: DirEntry,
    directory: &mut Directory,
) -> Result<()> {
    let file_name = entry.file_name().to_string_lossy().to_string();
    let entry_relative_path = relative_path.clone().join(&file_name);

    // Recursively discover new directories
    let path = entry.path();
    if path.is_dir() {
        let sub_directory = read_directory(&root, &entry_relative_path)?;
        directory.entries.push(Entry::Directory(sub_directory));
    } else if path.is_file() {
        let mut file_content = read_to_string(&path)
            .map_err(|err| Error::IoPathError(path.clone(), "reading file at", err))?;

        let mut file_config: Option<FileConfig> = None;

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
