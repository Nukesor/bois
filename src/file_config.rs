use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pub owner: String,
    pub group: String,
    /// This is represented as a octal `Oo755` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub permissions: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DirectoryConfig {
    pub owner: String,
    pub group: String,
    /// This is represented as a octal `Oo755` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub permissions: u32,
}
