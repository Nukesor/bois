//! The main configuration file, that's used to configure this program.
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use log::info;
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

use crate::error::Error;

/// The current mode we're running in.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub enum Mode {
    User,
    System,
}

/// All settings which are used by the daemon
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// The name of the machine.
    /// If this is set to None, the hostname will be used
    #[serde(default = "default_hostname")]
    pub name: String,

    /// The bois directory, which contains all bois templates and alike.
    /// This must be a path to an existing directory.
    bois_dir: PathBuf,

    /// The target directory to which the files should be deployed.
    /// This must be a path to an existing directory.
    target_dir: PathBuf,

    /// This allows you to set additional environment variables.
    /// This is mostly necessary for password manager integration, which need special
    /// configuration or get their sessions via environment variables.
    #[serde(default)]
    pub envs: HashMap<String, String>,
}

fn default_hostname() -> String {
    let result = hostname::get();

    let hostname = match result {
        Ok(hostname) => hostname,
        Err(err) => panic!(
            "Failed to determine hostname for machine: {err}
If this doesn't work, set the machine's name manually in the global bois.yml."
        ),
    };

    hostname.to_string_lossy().to_string()
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            name: default_hostname(),
            bois_dir: PathBuf::from("/etc/bois/"),
            target_dir: PathBuf::from("/"),
            envs: HashMap::new(),
        }
    }
}

/// Little helper which expands a given path's `~` characters to a fully qualified path.
pub fn expand_home(old_path: &Path) -> PathBuf {
    PathBuf::from(tilde(&old_path.to_string_lossy()).into_owned())
}

impl Configuration {
    /// The config directory which contains all bois templates and alike.
    pub fn bois_dir(&self) -> PathBuf {
        expand_home(&self.bois_dir)
    }

    /// The target directory to which the configuration should be deployed.
    pub fn target_dir(&self) -> PathBuf {
        expand_home(&self.target_dir)
    }
}

impl Configuration {
    /// Try to read existing config files, while using default values for non-existing fields.
    /// If successful, this will return a full config as well as a boolean on whether we found an
    /// existing configuration file or not.
    ///
    /// The default local config locations depends on the current target.
    pub fn read(from_file: &Option<PathBuf>) -> Result<(Configuration, bool)> {
        info!("Parsing config files");

        // Load the config from a very specific file path
        let path = if let Some(path) = from_file {
            let path = expand_home(path);
            if !path.exists() || !path.is_file() {
                bail!("Cannot find configuration file at path {path:?}");
            }

            path.clone()
        } else {
            // Get the default path for the user's configuration directory.
            let config_dir = PathBuf::from("/etc/bois");
            let path = config_dir.join("bois.yml");
            info!("Looking for config at path: {path:?}");

            // Use the default path, if we cannot find any file.
            if !path.exists() || !path.is_file() {
                info!("No config file found. Use default config.");
                // Return a default configuration if we couldn't find a file.
                return Ok((Configuration::default(), false));
            };

            path
        };

        info!("Found config file at: {path:?}");

        // Open the file in read-only mode with buffer.
        let file =
            File::open(&path).map_err(|err| Error::IoPath(path, "opening config file.", err))?;
        let reader = BufReader::new(file);

        // Read and deserialize the config file.
        let config: Configuration = serde_yaml::from_reader(reader)
            .map_err(|err| Error::ConfigDeserialization(err.to_string()))?;

        // Do some basic sanity checks
        if !config.target_dir().exists() {
            log::error!(
                "Bois target directory doesn't exist: {:?}",
                config.target_dir()
            );
            bail!("Target directory not found");
        }

        if !config.bois_dir().exists() {
            log::error!(
                "Bois config directory doesn't exist: {:?}",
                config.bois_dir()
            );
            bail!("Config directory not found");
        }

        Ok((config, true))
    }

    /// Save the current configuration as a file to the given path. \
    /// If no path is given, the default configuration path will be used. \
    /// The file is then written to the main configuration directory of the respective OS.
    pub fn save(&self, path: &Option<PathBuf>) -> Result<(), Error> {
        let config_path = if let Some(path) = path {
            path.clone()
        } else {
            PathBuf::from("/etc/bois/bois.yml")
        };

        let config_dir = PathBuf::from("/etc/bois");

        // Create the config dir, if it doesn't exist yet
        if !config_dir.exists() {
            create_dir_all(&config_dir)
                .map_err(|err| Error::IoPath(config_dir.clone(), "creating config dir", err))?;
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
        let mut file = File::create(config_path)
            .map_err(|err| Error::IoPath(config_dir.clone(), "creating settings file", err))?;
        file.write_all(content.as_bytes())
            .map_err(|err| Error::IoPath(config_dir, "writing settings file", err))?;

        Ok(())
    }
}
