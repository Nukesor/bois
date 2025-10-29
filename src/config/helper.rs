use std::{
    fs::{create_dir_all, read_to_string},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use log::trace;
use serde::de::DeserializeOwned;
use shellexpand::tilde;

use crate::error::Error;

/// Little helper which expands a given path's `~` characters to a fully qualified path.
pub fn expand_home(old_path: &Path) -> PathBuf {
    PathBuf::from(tilde(&old_path.to_string_lossy()).into_owned())
}

/// Look for a specific directory at a set of given paths.
/// Return the first directory that can be found.
///
/// If no directory can be found at any of the locations, throw an error message.
pub fn find_directory(
    dirs: Vec<Option<PathBuf>>,
    dir_type: &'static str,
    create_on_missing: bool,
) -> Result<PathBuf> {
    // First up, filter all None values from the Vector.
    // Those may show up if the home/config directory cannot be determined.
    let dirs: Vec<PathBuf> = dirs.into_iter().flatten().collect();

    // Check if any of the directories we're looking for exists.
    for dir in &dirs {
        if dir.exists() {
            return Ok(dir.clone());
        }
    }

    // If this is one of the directories we should create (such as runtime/cache dirs), we just
    // use the first directory as the default and do so.
    if create_on_missing {
        if let Some(default_dir) = dirs.first() {
            create_dir_all(default_dir).map_err(|err| {
                Error::IoPathString(default_dir.clone(), format!("creating {dir_type} dir"), err)
            })?;
            return Ok(default_dir.clone());
        }
    }

    // In case we expect at least one of the directories to already exist, but none of the paths
    // actually exist, we throw a detailed error message.
    let dir_strings: Vec<String> = dirs
        .into_iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();

    bail!(
        "Couldn't find {dir_type} directory in one of these locations:\n{}",
        dir_strings.join("\n")
    );
}

pub fn read_yaml<T: DeserializeOwned>(directory: &Path, filename: &str) -> Result<T, Error> {
    let mut path = directory.join(filename);
    trace!("Read yaml file at {path:?}.y[a]ml");
    // Check if the file exists with `yml` or `yaml` extension.
    path.set_extension("yml");
    if !path.exists() {
        path.set_extension("yaml");
        if !path.exists() {
            return Err(Error::FileNotFound(
                format!("{filename}.y[a]ml"),
                directory.to_path_buf(),
            ));
        }
    }

    let content =
        read_to_string(&path).map_err(|err| Error::IoPath(path.clone(), "reading", err))?;

    serde_yaml::from_str::<T>(&content).map_err(|err| Error::Deserialization(path.clone(), err))
}
