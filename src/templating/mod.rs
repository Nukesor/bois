use std::fs::{self, read_to_string};
use std::path::PathBuf;

use anyhow::{Context, Result};

use pest::Parser;

use crate::error::Error;
use crate::parser;

pub mod file;
pub mod prerender_state;
pub mod state;

pub fn discover_files(root: &PathBuf, relative_path: &PathBuf) -> Result<file::Directory> {
    let directory_path = root.join(relative_path);
    let entries = fs::read_dir(&directory_path)
        .map_err(|err| Error::IoPathError(directory_path.clone(), "reading", err))?;

    // Create the representation for this directory
    let mut directory = file::Directory {
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
            directory
                .entries
                .push(file::Entry::Directory(sub_directory));
        } else if path.is_file() {
            let mut file_content = read_to_string(&path)
                .map_err(|err| Error::IoPathError(path.clone(), "reading file at", err))?;

            let mut file_config: Option<file::FileConfig> = None;
            // We found a start keyword for a bois configuration block.
            if file_content.contains("bois_config_start") {
                // Parse the bois configuration block
                let mut parsed =
                    parser::ConfigParser::parse(parser::Rule::full_config, &file_content)
                        .context(format!("Failed to parse config block in file at {path:?}"))?;

                // The first parsed block is the bois configuration.
                let config_text = parsed.next().unwrap().to_string();
                // The second parsed block is the actual content of the file.
                file_content = parsed.next().unwrap().to_string();
                dbg!(&config_text);

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
    }

    Ok(directory)
}
