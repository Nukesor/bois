use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, Write},
};

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{
    config::{bois::Configuration, services::Service},
    error::Error,
    state::path::tree::Tree,
};

pub mod path;

#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Display, Deserialize, Serialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "PascalCase")]
pub enum PackageManager {
    Pacman,
    Paru,
    Apt,
}

#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Display, Deserialize, Serialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ServiceManager {
    Systemd,
}

/// This struct all configuration that's applicable for this machine.
/// This includes:
/// - All applicable traits
///     - Variables
///     - Directories
///     - Files/Templates
///     - In-file and in-directory configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    /// The current configuration
    /// We have to save this as well, as we would otherwise loose information about
    /// previous runs, if the config changed in the meantime such as, for instance, a different
    /// root dir or new hostname.
    pub configuration: Configuration,

    /// The full tree of all configuration files as they are read from the configuration directory,
    pub path_tree: Tree,

    /// The compiled list of all packages that should be installed for this current configuration.
    pub packages: BTreeMap<PackageManager, BTreeSet<String>>,

    /// The compiled list of all services that should be enabled for this current configuration.
    #[serde(default)]
    pub services: BTreeMap<ServiceManager, BTreeSet<Service>>,
}

impl State {
    /// Try to read the state of a previous deployment.
    /// This state will be used to determine:
    /// - Any changes on the system's files since the last deployment
    /// - Cleanup work that might need to be done for the new desired state.
    ///
    /// Will return a Ok(None), if no previous state could be found.
    pub fn read_previous(config: &Configuration) -> Result<Option<Self>> {
        // Get the path for the deployed state.
        let path = config.cache_dir.join("deployed_state.yml");
        info!("Looking for previous state file at {path:?}");

        // Return None if we cannot find any file.
        if !path.exists() || !path.is_file() {
            info!("No state file found. Use default config.");
            return Ok(None);
        };

        info!("Found previous deployed state at: {path:?}");

        // Open the file in read-only mode with buffer.
        let file = File::open(&path)
            .map_err(|err| Error::IoPath(path.clone(), "opening config file.", err))?;
        let reader = BufReader::new(file);

        // Read and deserialize the config file.
        let state =
            serde_yaml::from_reader(reader).map_err(|err| Error::Deserialization(path, err))?;

        Ok(state)
    }

    /// Save the current desired state as a file. \
    /// Read the `self.read` docs on why we need this file at all.
    pub fn save(&self) -> Result<(), Error> {
        let path = self.configuration.cache_dir.join("deployed_state.yml");
        info!("Looking for previous state file at {path:?}");

        // Serialize the configuration file and write it to disk
        let content = match serde_yaml::to_string(self) {
            Ok(content) => content,
            Err(error) => {
                return Err(Error::Generic(format!(
                    "Configuration file serialization failed:\n{error}"
                )));
            }
        };

        // Write the serialized content to the file.
        let mut file = File::create(&path)
            .map_err(|err| Error::IoPath(path.clone(), "creating state file", err))?;
        file.write_all(content.as_bytes())
            .map_err(|err| Error::IoPath(path, "writing state file", err))?;

        Ok(())
    }
}
