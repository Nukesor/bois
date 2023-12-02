use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

// /// Paths are usually relative to a given root directory, but it's also possible to specify
// /// absolute paths that aren't necessarily in the specified root directory.
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub enum Path {
//     Relative(PathBuf),
//     Absolute(PathBuf),
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Entry {
    File(File),
    Directory(Directory),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pub owner: String,
    pub group: String,
    /// This is represented as a octal `Oo755` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub permissions: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct File {
    /// The path to the source file.
    /// Relative to the root directory of the configuration.
    pub path: PathBuf,

    /// The parsed configuration block for this file, if one exists.
    pub config: Option<FileConfig>,

    /// The configuration file's content, without the bois configuration block.
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DirectoryConfig {
    pub owner: String,
    pub group: String,
    /// This is represented as a octal `Oo755` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub permissions: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Directory {
    pub path: PathBuf,
    pub entries: Vec<Entry>,
    pub config: Option<DirectoryConfig>,
}
