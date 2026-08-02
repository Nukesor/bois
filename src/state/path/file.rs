use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};

use crate::state::path::tree::Source;

/// A fully resolved file as it should end up on the system.
///
/// All configuration (defaults cascade, in-file config block, templating) has
/// already been applied during aggregation.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct FileState {
    /// The file's content, with the `bois_config` block stripped and templating applied.
    pub content: FileContent,

    /// The permissions the file should have on the system (12 bit, no filetype bits).
    pub mode: u32,

    /// The user that should own the file.
    pub owner: String,

    /// The group that should own the file.
    pub group: String,

    /// Used to keep track of which group/host and path this file originated from.
    /// We need this to show conflict errors and display diffs.
    pub source: Source,
}

/// The file-type to abstract
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum FileContent {
    Text(String),
    /// Binary content, serialized as base64 in the state file.
    ///
    /// It's not optimal and increases size by ~33%, but this has to do
    /// unless we switch to a more efficient data format than YAML.
    Binary(#[serde(with = "base64_bytes")] Vec<u8>),
}

impl FileContent {
    /// Get the file content as bytes independent of type.
    pub fn bytes(&self) -> &[u8] {
        match self {
            FileContent::Text(text) => text.as_bytes(),
            FileContent::Binary(bytes) => bytes,
        }
    }
}

mod base64_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::*;

    pub fn serialize<S: Serializer>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&BASE64_STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let encoded = String::deserialize(deserializer)?;
        BASE64_STANDARD
            .decode(encoded)
            .map_err(serde::de::Error::custom)
    }
}
