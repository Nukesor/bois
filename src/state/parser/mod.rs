use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use log::debug;
use pest::Parser;
use pest_derive::Parser;

use super::file::{File, FileConfig};
use crate::error::Error;

#[derive(Parser)]
#[grammar = "state/parser/syntax.pest"]
pub struct ConfigParser;

// TODO: Try using the ConfigParser again.
pub fn read_file_with_parser(path: PathBuf) -> Result<File> {
    let mut file_content =
        read_to_string(&path).map_err(|err| Error::IoPath(path.clone(), "reading file at", err))?;

    let mut file_config = FileConfig::default();

    // Check if there's the key word for a in-file configuration block.
    // If so, try to parse the file as such.
    if file_content
        .lines()
        .next()
        .map(|line| line.contains("bois_config_start"))
        .unwrap_or(false)
    {
        // Parse the bois configuration block
        let mut parsed = ConfigParser::parse(Rule::full_config, &file_content)
            .context(format!("Failed to parse config block in file at {path:?}"))?;

        // The first parsed block is the bois configuration.
        let config_text = parsed
            .next()
            .context(format!(
                "Failed to read inline bois configuration from file {path:?}"
            ))?
            .to_string();
        // The second parsed block is the actual content of the file, without the bois config.
        file_content = parsed
            .next()
            .context(format!(
                "Failed to get parsed normal content for file {path:?}"
            ))?
            .to_string();
        debug!("Found config block in file {path:?}:\n{config_text}");

        // Try to deserialize the bois configuration content into the correct struct.
        file_config = serde_yaml::from_str(&config_text).context(format!(
            "Failed to deserialize bois config inside of file {path:?}"
        ))?;
    }

    // Create a new representation of a file with all necessary information.
    Ok(File {
        relative_path: path,
        config: file_config,
        content: file_content,
    })
}

pub fn parse_file(root: &Path, relative_path: &Path) -> Result<File> {
    let path = root.join(relative_path);

    let full_file_content =
        read_to_string(&path).map_err(|err| Error::IoPath(path.clone(), "reading file at", err))?;

    // Check, if the first line of the file contains the bois_config_start keyword.
    // If so, there's a bois config block in that file and we have to parse it.
    let contains_config = {
        let mut lines_iter = full_file_content.lines();
        lines_iter
            .next()
            .map(|line| line.contains("bois_config_start"))
            .unwrap_or(false)
    };

    // If there's no config, there's nothing to do and we can just return the file with the default
    // FileConfig straight away.
    if !contains_config {
        return Ok(File {
            relative_path: path,
            content: full_file_content,
            config: FileConfig::default(),
        });
    }

    // If we have a config block.
    // 1. Take all lines between `bois_config_start` and `bois_config_end`. For each line
    //   - Strip any comment trailing spaces
    //   - Strip any comment symbols
    // 2. Deserialize the config

    // Create an iterator over lines, as we have to read and clean up lines until we hit
    // `bois_config_end`.
    let mut lines_iter = full_file_content.lines();
    // Skip the first line, since we know that it's the bois_config_start line.
    lines_iter.next();

    let mut config_complete = false;
    let mut config_content: Vec<String> = Vec::new();
    for line in lines_iter.by_ref() {
        if line.contains("bois_config_end") {
            config_complete = true;
            break;
        }

        // Remove any trailing spaces and comment symbols/sequences.
        let mut line = line.trim();
        if line.starts_with("//") {
            line = line.strip_prefix("//").unwrap_or_default();
        } else if line.starts_with("--") {
            line = line.strip_prefix("--").unwrap_or_default();
        } else if line.starts_with("*/") {
            line = line.strip_prefix("*/").unwrap_or_default();
        } else if line.starts_with("/*") {
            line = line.strip_prefix("/*").unwrap_or_default();
        } else if line.starts_with("**") {
            line = line.strip_prefix("**").unwrap_or_default();
        } else if line.starts_with('*') {
            line = line.strip_prefix('*').unwrap_or_default();
        } else if line.starts_with('#') {
            line = line.strip_prefix('#').unwrap_or_default();
        } else if line.starts_with('%') {
            line = line.strip_prefix('%').unwrap_or_default();
        }

        config_content.push(line.to_string());
    }

    if !config_complete {
        bail!("Didn't encounter 'bois_config_end' block while reading file {path:?}");
    }

    debug!("Found config block in file {path:?}:\n{config_content:#?}");
    let config = serde_yaml::from_str(&config_content.join("\n"))?;

    // Now, read the rest of the actual
    let content = lines_iter.collect::<Vec<&str>>().join("\n");

    Ok(File {
        relative_path: relative_path.to_path_buf(),
        config,
        content,
    })
}
