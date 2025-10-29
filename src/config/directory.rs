use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    config::helper::expand_home,
    constants::{CURRENT_GROUP, CURRENT_USER},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryConfig {
    /// If this is set, this directory will be used as the default destination to write configs to.
    /// - If it's an relative path, it'll be treated as relative to the default target directory.
    /// - If it's an absolute path, that absolute path will be used.
    path: Option<PathBuf>,
    pub owner: Option<String>,
    pub group: Option<String>,
    /// This is represented as a octal `Oo755` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub mode: Option<u32>,
}

/// This impl block contains convenience getters for directory metadata, which fall back to
/// default values.
impl DirectoryConfig {
    pub fn path(&self) -> Option<PathBuf> {
        self.path.as_ref().map(|path| expand_home(path))
    }

    pub fn override_path(&mut self, path: PathBuf) {
        self.path = Some(path)
    }

    pub fn mode(&self) -> u32 {
        self.mode.unwrap_or(0o755)
    }

    pub fn owner(&self) -> String {
        self.owner.clone().unwrap_or(CURRENT_USER.clone())
    }

    pub fn group(&self) -> String {
        self.group.clone().unwrap_or(CURRENT_GROUP.clone())
    }
}
