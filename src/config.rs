//! The main configuration file, that's used to configure this program.
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

use anyhow::{bail, Result};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    helper::{expand_home, find_directory},
};

/// The current mode we're running in.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Mode {
    User,
    System,
}

/// This
#[derive(PartialEq, Eq, Clone, Default, Debug, Deserialize, Serialize)]
pub struct RawConfiguration {
    /// The name of the machine.
    /// If this is set to None, the hostname will be used
    pub name: Option<String>,

    /// The bois directory, which contains all bois templates and alike.
    /// This must be a path to an existing directory.
    pub bois_dir: Option<PathBuf>,

    /// The target directory to which the files should be deployed.
    /// This must be a path to an existing directory.
    pub target_dir: Option<PathBuf>,

    /// Cache dir
    pub cache_dir: Option<PathBuf>,

    /// Runtime dir
    pub runtime_dir: Option<PathBuf>,

    /// This allows you to set additional environment variables.
    /// This is mostly necessary for password manager integration, which need special
    /// configuration or get their sessions via environment variables.
    #[serde(default)]
    pub envs: HashMap<String, String>,

    /// Determine whether
    pub mode: Option<Mode>,
}

/// All settings which are used by the daemon
/// TODO: Create two different configuration structs.
///     One for parsing the file and one that's populated with the default values afterwards.
///     This will enable us to have a clean fully-populated Configuration struct for our
///     application logic and a lazy incomplete struct for parsing.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    /// The name of the machine.
    /// If this is set to None, the hostname will be used
    pub name: String,

    /// The bois directory, which contains all bois templates and alike.
    /// This must be a path to an existing directory.
    pub bois_dir: PathBuf,

    /// The target directory to which the files should be deployed.
    /// This must be a path to an existing directory.
    pub target_dir: PathBuf,

    /// Cache dir
    pub cache_dir: PathBuf,

    /// Runtime dir
    pub runtime_dir: PathBuf,

    /// This allows you to set additional environment variables.
    /// This is mostly necessary for password manager integration, which need special
    /// configuration or get their sessions via environment variables.
    pub envs: HashMap<String, String>,

    /// Determine whether bois is running in system configuration mode or in
    /// user configuration mode.
    pub mode: Mode,
}

impl RawConfiguration {
    /// The mode in which bois operates (root vs. user).
    /// Fallback to `Mode::System` for now, will be changed later.
    /// This determines a few things, such as:
    /// - Whether `systemctl` should be called with the `--user` flag
    /// - What the default target directory is `~/.config` vs `/etc`
    pub fn mode(&self) -> Mode {
        self.mode.unwrap_or(Mode::System)
    }
}

/// This function takes a [RawConfiguration] from a deserialized config file and populates all values
/// that haven't explicitly set.
/// The resulting [Configuration] no longer has any `Option`als, which makes it convenient to pass
/// around the program during runtime.
pub fn build_configuration(raw_config: RawConfiguration) -> Result<Configuration> {
    // Determine the hostname of the machine, if it isn't explicitly set.
    let name = match raw_config.name {
        Some(name) => name,
        None => match hostname::get() {
            Ok(hostname) => hostname.to_string_lossy().to_string(),
            Err(err) => bail!(
                "Failed to determine hostname for machine: {err}
If this doesn't work, set the machine's name manually in the global bois.yml."
            ),
        },
    };

    // Determine the mode bois should run in.
    // This determines what kind of default directories should be used, which is why we do it very
    // early on.
    let mode = match raw_config.mode {
        Some(mode) => mode,
        None => {
            if whoami::username() == "root" {
                Mode::System
            } else {
                Mode::User
            }
        }
    };

    // Determine the directory where we should look for the config files.
    let bois_dir = match raw_config.bois_dir {
        Some(dir) => expand_home(&dir),
        None => match mode {
            Mode::User => find_directory(
                vec![
                    dirs::config_dir().map(|path| path.join("dotfiles")),
                    dirs::config_dir().map(|path| path.join("bois")),
                    dirs::home_dir().map(|path| path.join(".dotfiles")),
                    dirs::home_dir().map(|path| path.join(".dots")),
                    dirs::home_dir().map(|path| path.join(".bois")),
                ],
                "bois config",
                false,
            )?,
            Mode::System => PathBuf::from("/etc/bois"),
        },
    };

    // Determine the directory where we should look for the config files.
    let target_dir = match raw_config.target_dir {
        Some(dir) => expand_home(&dir),
        None => match mode {
            Mode::User => find_directory(
                vec![
                    dirs::config_dir(),
                    dirs::home_dir().map(|path| path.join(".config")),
                ],
                "target",
                true,
            )?,
            Mode::System => PathBuf::from("/etc/bois"),
        },
    };

    // Determine the directory where we store cached files, such as the previous deployed state.
    let cache_dir = match raw_config.cache_dir {
        Some(dir) => expand_home(&dir),
        None => match mode {
            Mode::User => find_directory(
                vec![dirs::cache_dir().map(|path| path.join("bois"))],
                "bois cache",
                true,
            )?,
            Mode::System => PathBuf::from("/var/lib/bois"),
        },
    };

    // Determine the directory where we store cached files, such as the previous deployed state.
    let runtime_dir = match raw_config.runtime_dir {
        Some(dir) => expand_home(&dir),
        None => match mode {
            Mode::User => find_directory(
                vec![
                    dirs::runtime_dir().map(|path| path.join("bois")),
                    dirs::cache_dir().map(|path| path.join("bois")),
                ],
                // If we cannot detect a runtime dir, fallback to the cache dir.
                "bois runtime",
                true,
            )?,
            Mode::System => PathBuf::from("/var/lib/bois"),
        },
    };

    Ok(Configuration {
        name,
        bois_dir,
        target_dir,
        cache_dir,
        runtime_dir,
        envs: raw_config.envs,
        mode,
    })
}

impl RawConfiguration {
    /// Try to read existing config files, while using default values for non-existing fields.
    /// If successful, this will return a full config as well as a boolean on whether we found an
    /// existing configuration file or not.
    ///
    /// The default local config locations depends on the current target.
    pub fn read(from_file: &Option<PathBuf>) -> Result<RawConfiguration> {
        info!("Parsing config files");

        // Load the config from a very specific file path
        let path = if let Some(path) = from_file {
            let path = expand_home(path);
            if !path.exists() || !path.is_file() {
                bail!("Cannot find configuration file at path {path:?}");
            }

            path.clone()
        } else {
            // If bois is running as root, we assume that it's used te
            let config_dir = if whoami::username() == "root" {
                PathBuf::from("/etc/bois")
            } else {
                find_directory(
                    vec![
                        dirs::config_dir().map(|path| path.join("dotfiles")),
                        dirs::config_dir().map(|path| path.join("bois")),
                        dirs::home_dir().map(|path| path.join(".dotfiles")),
                        dirs::home_dir().map(|path| path.join(".dots")),
                        dirs::home_dir().map(|path| path.join(".bois")),
                    ],
                    "bois config",
                    false,
                )?
            };

            // Get the default path for the user's configuration directory.
            let path = config_dir.join("bois.yml");
            info!("Looking for config at path: {path:?}");

            // Use the default path, if we cannot find any file.
            if !path.exists() || !path.is_file() {
                info!("No config file found. Use default config.");
                // Return a default configuration if we couldn't find a file.
                return Ok(RawConfiguration::default());
            };

            path
        };

        info!("Found config file at: {path:?}");

        // Open the file in read-only mode with buffer.
        let file = File::open(&path)
            .map_err(|err| Error::IoPath(path.clone(), "opening config file.", err))?;
        let reader = BufReader::new(file);

        // Read and deserialize the config file.
        let config: RawConfiguration =
            serde_yaml::from_reader(reader).map_err(|err| Error::Deserialization(path, err))?;

        Ok(config)
    }

    // TODO: Commented out for now. May delete later.
    // /// Save the current configuration as a file to the given path. \
    // /// If no path is given, the default configuration path will be used. \
    // /// The file is then written to the main configuration directory of the respective OS.
    //pub fn save(&self, path: &Option<PathBuf>) -> Result<(), Error> {
    //    let config_path = if let Some(path) = path {
    //        path.clone()
    //    } else {
    //        PathBuf::from("/etc/bois/bois.yml")
    //    };

    //    let config_dir = PathBuf::from("/etc/bois");

    //    // Create the config dir, if it doesn't exist yet
    //    if !config_dir.exists() {
    //        create_dir_all(&config_dir)
    //            .map_err(|err| Error::IoPath(config_dir.clone(), "creating config dir", err))?;
    //    }

    //    // Serialize the configuration file and write it to disk
    //    let content = match serde_yaml::to_string(self) {
    //        Ok(content) => content,
    //        Err(error) => {
    //            return Err(Error::Generic(format!(
    //                "Configuration file serialization failed:\n{error}"
    //            )))
    //        }
    //    };

    //    // Write the serialized content to the file.
    //    let mut file = File::create(config_path)
    //        .map_err(|err| Error::IoPath(config_dir.clone(), "creating settings file", err))?;
    //    file.write_all(content.as_bytes())
    //        .map_err(|err| Error::IoPath(config_dir, "writing settings file", err))?;

    //    Ok(())
    //}
}
