use pest::Parser;
use std::fs::{self, read_to_string, DirEntry};
use std::path::PathBuf;

use anyhow::{Context, Result};
use log::debug;

use crate::{
    error::Error,
    file_config::{DirectoryConfig, FileConfig},
};
use parser::{ConfigParser, Rule};

pub mod file;
mod parser;
pub mod state;

pub fn discover_files(root: &PathBuf, relative_path: &PathBuf) -> Result<file::Directory> {
    let directory_path = root.join(relative_path);
    let entries = fs::read_dir(&directory_path)
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
    let mut directory = file::Directory {
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
    directory: &mut file::Directory,
) -> Result<()> {
    let file_name = entry.file_name().to_string_lossy().to_string();
    let entry_relative_path = relative_path.clone().join(&file_name);

    // Recursively discover new directories
    let path = entry.path();
    if path.is_dir() {
        let sub_directory = discover_files(&root, &entry_relative_path)?;
        directory
            .entries
            .push(file::Entry::Directory(sub_directory));
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
        let file = file::File {
            path,
            config: file_config,
            content: file_content,
        };

        // Add it to the current directory.
        directory.entries.push(file::Entry::File(file));
    }

    Ok(())
}
