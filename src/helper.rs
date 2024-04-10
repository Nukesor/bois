use std::{fs::read_to_string, path::Path};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

use crate::error::Error;

pub fn read_yaml<T: DeserializeOwned>(directory: &Path, filename: &str) -> Option<Result<T>> {
    let mut path = directory.join(filename);
    // Check if the file exists with `yml` or `yaml` extension.
    path.set_extension("yml");
    if !path.exists() {
        path.set_extension("yaml");
        if !path.exists() {
            return None;
        }
    }

    let content =
        read_to_string(&path).map_err(|err| Error::IoPathError(path.clone(), "reading", err));
    let content = match content {
        Ok(content) => content,
        Err(err) => return Some(Err(anyhow::anyhow!(err))),
    };

    Some(
        serde_yaml::from_str::<T>(&content)
            .context(format!("Failed serializing config at path: {path:?}")),
    )
}
