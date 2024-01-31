use std::collections::HashMap;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::config::Configuration;

pub mod file;
mod group;
mod parser;

use group::Group;

use self::group::read_group;

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

impl State {
    pub fn new(config: &Configuration) -> Result<Self> {
        let mut groups = Vec::new();
        let hostgroup = read_group(&config.bois_dir(), &config.name()?)?;

        for group_name in &hostgroup.dependencies {
            let group = read_group(&config.bois_dir(), group_name)?;
            groups.push(group);
        }

        groups.push(hostgroup);

        Ok(State {
            groups,
            global_variables: HashMap::new(),
        })
    }
}
