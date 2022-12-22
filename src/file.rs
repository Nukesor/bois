use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct File {
    /// The path to the source file.
    /// Relative to the root directory of the configuration.
    path: PathBuf,
    /// The relative destination path
    destination: PathBuf,

    /// The parsed configuration for this file.
    config: FileConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pre_hooks: Vec<String>,
    post_hooks: Vec<String>,
    permissions: String,
}
