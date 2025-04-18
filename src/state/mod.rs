use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Write},
};

use anyhow::{Result, bail};
use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    config::Configuration,
    error::Error,
    handlers::packages::{PackageManager, pacman::get_packages_for_group},
    system_state::SystemState,
};

pub mod directory;
pub mod file;
pub mod file_parser;
pub mod group;
pub mod host;

use self::{
    group::read_group,
    host::{Host, read_host},
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
    pub variables: HashMap<String, String>,
    /// The current configuration
    /// We have to save this as well, as we would otherwise loose information about
    /// previous runs, if the config changed in the meantime such as, for instance, a different
    /// root dir or new hostname.
    pub configuration: Configuration,

    /// The compiled list of all packages that should be installed for this current configuration.
    pub packages: HashMap<PackageManager, HashSet<String>>,
}

impl State {
    /// Build a new state from a current bois configuration.
    /// This state only represents the desired state for the **current** machine.
    pub fn new(configuration: &Configuration, system_state: &mut SystemState) -> Result<Self> {
        // Check whether the most important directories are present as expected.
        let bois_dir = configuration.bois_dir.clone();
        if !bois_dir.exists() {
            eprintln!("Couldn't find bois config directory at {bois_dir:?}. Aborting.");
            bail!("Couldn't find config directory.");
        }

        // Read the initial group for this host.
        // This specifieds all other dependencies.
        let mut host = read_host(&configuration.bois_dir, &configuration.name)?;

        // Go through all dependencies and load them as well.
        for group_name in &host.config.groups {
            let group = read_group(&configuration.bois_dir, group_name, &host.variables)?;
            host.groups.push(group);
        }

        let mut state = State {
            host,
            variables: HashMap::new(),
            configuration: configuration.clone(),
            packages: HashMap::new(),
        };

        state.load_packages(system_state)?;

        Ok(state)
    }

    /// Check whether there're any duplicate packages for a given package manager.
    fn load_packages(&mut self, system_state: &mut SystemState) -> Result<()> {
        // Check all host packages.
        for (manager, packages) in self.host.config.packages.iter() {
            let known_packages = self.packages.entry(*manager).or_default();

            // Print a warning for all duplicate packages
            for duplicate in packages.intersection(known_packages) {
                warn!("Found duplicate package {duplicate} in host.yml");
            }

            known_packages.extend(packages.clone());
        }

        // Check all group packages.
        for group in self.host.groups.iter() {
            for (manager, packages) in group.config.packages.iter() {
                let known_packages = self.packages.entry(*manager).or_default();

                // Print a warning for all duplicate packages
                for duplicate in packages.intersection(known_packages) {
                    warn!(
                        "Found duplicate package {duplicate} in group.yml for group {}",
                        group.name
                    );
                }

                known_packages.extend(packages.clone());
            }
        }

        // If there're any groups for any of the package managers, unroll the group and remove the
        // group from the list off packages to install.
        for (manager, ref mut packages) in self.packages.iter_mut() {
            let mut detected_groups = HashSet::new();
            let mut group_packages = HashSet::new();
            let groups_on_system = system_state.detected_package_groups(*manager)?;

            for name in packages.iter() {
                // Check if any package is a group
                if groups_on_system.contains(name) {
                    // Safe the group and its packages so we can fix the package list.
                    detected_groups.insert(name.clone());
                    group_packages.extend(get_packages_for_group(name)?)
                }
            }

            // Add all packages from detected groups.
            packages.extend(group_packages);
            // Remove all groups.
            packages.retain(|name| !detected_groups.contains(name));
        }

        Ok(())
    }

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
