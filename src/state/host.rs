use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use super::directory::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Group {
    /// The top-level configuration file for this group/host.
    pub group_config: GroupConfig,
    /// The content of this group's directory.
    pub directory: Directory,
    /// The content of this group's directory.
    pub defaults: GroupDefaults,
    /// All variables that're available to all groups during templating.
    pub global_variables: HashMap<String, String>,
    /// All variables that're available during templating for this group.
    pub variables: HashMap<String, String>,
    /// All other groups that should be loaded.
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GroupConfig {
    /// Other groups that're required by this group.
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GroupDefaults {
    pub owner: Option<String>,
    pub group: Option<String>,
}

pub fn read_group(root: &Path, name: &str) -> Result<Group> {
    let directory = read_directory(&root.join(name), &PathBuf::new())?;

    Ok(Group {
        directory,
        ..Default::default()
    })
}
