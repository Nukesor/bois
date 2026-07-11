use serde::{Deserialize, Serialize};

use crate::config::file::FileConfig;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct File {
    /// The metadata of the source file.
    /// Used to determine the mode in case it isn't overwritten.
    pub mode: u32,

    /// The parsed configuration block for this file, if one exists.
    #[serde(default)]
    pub config: FileConfig,

    /// The actual configuration file's content, without the bois configuration block.
    /// and fully templated.
    pub content: String,
}
