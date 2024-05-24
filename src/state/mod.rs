use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir_all, File},
    io::{BufReader, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use log::{info, warn};
use serde_derive::{Deserialize, Serialize};

use crate::{config::Configuration, error::Error, handlers::packages::PackageManager};

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
#[derive(Clone, Debug, Serialize, Deserialize)]
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

    /// Run a few sanity checks on a given state. Such check include:
    /// - Check for duplicate package declarations
    pub fn lint(&self) {
        self.lint_packages();
    }

    /// Check whether there're any duplicate packages for a given package manager.
    fn lint_packages(&self) {
        // This list will contain all discovered packages.
        let mut all_packages: HashMap<PackageManager, HashSet<String>> = HashMap::new();

        // Check all host packages.
        for (manager, packages) in self.host.config.packages.iter() {
            let known_packages = all_packages.entry(*manager).or_insert(HashSet::new());

            // Print a warning for all duplicate packages
            for duplicate in packages.intersection(&known_packages) {
                warn!("Found duplicate package {duplicate} in host.yml");
            }
        }

        // Check all group packages.
        for group in self.host.groups.iter() {
            for (manager, packages) in group.config.packages.iter() {
                let known_packages = all_packages.entry(*manager).or_insert(HashSet::new());

                // Print a warning for all duplicate packages
                for duplicate in packages.intersection(&known_packages) {
                    warn!(
                        "Found duplicate package {duplicate} in group.yml for group {}",
                        group.name
                    );
                }
            }
        }
    }

    /// Try to read the state of a previous deployment.
    /// This state will be used to determine:
    /// - Any changes on the system's files since the last deployment
    /// - Cleanup work that might need to be done for the new desired state.
    ///
    /// Will return a Ok(None), if no previous state could be found.
    pub fn read_previous() -> Result<Option<Self>> {
        // Get the path for the deployed state.
        let application_dir = PathBuf::from("/var/lib/bois");
        let path = application_dir.join("deployed_state.yml");
        info!("Looking for previous state file at {path:?}");

        // Return None if we cannot find any file.
        if !path.exists() || !path.is_file() {
            info!("No state file found. Use default config.");
            return Ok(None);
        };

        info!("Found previous deployed state at: {path:?}");

        // Open the file in read-only mode with buffer.
        let file = File::open(&path)
            .map_err(|err| Error::IoPathError(path, "opening config file.", err))?;
        let reader = BufReader::new(file);

        // Read and deserialize the config file.
        let state = serde_yaml::from_reader(reader)
            .map_err(|err| Error::ConfigDeserialization(err.to_string()))?;

        Ok(state)
    }

    /// Save the current desired state as a file. \
    /// Read the `self.read` docs on why we need this file at all.
    pub fn save(&self) -> Result<(), Error> {
        let application_dir = PathBuf::from("/var/lib/bois");
        let path = application_dir.join("deployed_state.yml");
        info!("Looking for previous state file at {path:?}");

        // Create the dir, if it doesn't exist yet
        if !application_dir.exists() {
            create_dir_all(&application_dir).map_err(|err| {
                Error::IoPathError(application_dir.clone(), "creating state dir", err)
            })?;
        }

        // Serialize the configuration file and write it to disk
        let content = match serde_yaml::to_string(self) {
            Ok(content) => content,
            Err(error) => {
                return Err(Error::Generic(format!(
                    "Configuration file serialization failed:\n{error}"
                )))
            }
        };

        // Write the serialized content to the file.
        let mut file = File::create(path).map_err(|err| {
            Error::IoPathError(application_dir.clone(), "creating state file", err)
        })?;
        file.write_all(content.as_bytes())
            .map_err(|err| Error::IoPathError(application_dir, "writing state file", err))?;

        Ok(())
    }
}
