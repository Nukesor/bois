use std::collections::HashMap;

use anyhow::{bail, Result};
use serde_derive::{Deserialize, Serialize};

use crate::config::Configuration;

pub mod directory;
pub mod file;
pub mod group;
pub mod host;
mod parser;

use self::{
    group::read_group,
    host::{read_host, Host},
};

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
    pub host: Host,
    /// All variables that're available to all groups during templating.
    pub global_variables: HashMap<String, String>,
    /// The current configuration
    /// We have to save this as well, as we would otherwise loose information about
    /// previous runs, if the config changed in the meantime such as, for instance, a different
    /// root dir or new hostname.
    pub configuration: Configuration,
}

impl State {
    /// Build a new state from a current bois configuration.
    /// This state only represents the desired state for the **current** machine.
    pub fn new(configuration: Configuration) -> Result<Self> {
        // Check whether the most important directories are present as expected.
        let bois_dir = configuration.bois_dir();
        if !bois_dir.exists() {
            eprintln!("Couldn't find bois config directory at {bois_dir:?}. Aborting.");
            bail!("Couldn't find config directory.");
        }

        // Read the initial group for this host.
        // This specifieds all other dependencies.
        let mut host = read_host(&configuration.bois_dir(), &configuration.name()?)?;

        // Go through all dependencies and load them as well.
        for group_name in &host.config.dependencies {
            let group = read_group(&configuration.bois_dir(), group_name)?;
            host.groups.push(group);
        }

        Ok(State {
            host,
            global_variables: HashMap::new(),
            configuration,
        })
    }
}
