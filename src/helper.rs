use std::{fs::read_to_string, path::Path};

use log::trace;
use serde::de::DeserializeOwned;

use crate::error::Error;

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
