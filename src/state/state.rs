use std::collections::HashMap;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::{args::Arguments, group_config::GroupConfig};

use super::file::Directory;

/// This struct all configuration that's applicable for this machine.
/// This includes:
/// - All applicable groups
///     - Variables
///     - Directories
///     - Files/Templates
///     - In-file and in-directory configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    /// The diffent groups that're managed by bois.
    pub groups: Vec<Group>,
    /// All variables that're available to all groups during templating.
    pub global_variables: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    /// The top-level configuration file for this group/host.
    group_config: GroupConfig,
    /// The list of all top-level files/directories.
    directory: Directory,
    /// All variables that're available during templating for this group.
    variables: HashMap<String, String>,
}

impl State {
    pub fn new(args: &Arguments) -> Result<Self> {
        Ok(State {
            groups: Vec::new(),
            global_variables: HashMap::new(),
        })
    }
}
