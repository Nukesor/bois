use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::{directory::*, file::read_entry, group::Group};
use crate::{
    config::helper::read_yaml,
    error::Error,
    handlers::packages::PackageManager,
    templating::get_host_vars,
};

/// A Host is related to a
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Host {
    /// The top-level configuration file for this host.
    pub config: HostConfig,
    /// All variables that're available for templating to the host files and all groups.
    pub variables: Value,
    /// The content of this group's directory.
    pub directory: Directory,
    /// Will contain all groups that have been specified as dependencies.
    pub groups: Vec<Group>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostConfig {
    /// Used to overwrite the target directory to which files should be deployed for
    /// this specific group.
    #[serde(default)]
    pub target_directory: Option<PathBuf>,
    /// Default that should be applied to all files.
    #[serde(default)]
    pub file_defaults: HostDefaults,
    /// Groups that're required by this host.
    #[serde(default)]
    pub groups: Vec<String>,
    /// Packages that should always be installed for this host.
    #[serde(default)]
    pub packages: HashMap<PackageManager, HashSet<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HostDefaults {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub file_mode: Option<u32>,
    pub directory_mode: Option<u32>,
}

pub fn read_host(root: &Path, hostname: &str) -> Result<Host> {
    let host_dir = root.join("hosts").join(hostname);

    if !host_dir.exists() {
        eprintln!("Couldn't find config directory for this machine at {host_dir:?}. Aborting.");
        bail!("Couldn't find host config directory.");
    }

    // Read the `host.yml` from the host directory.
    let config = read_yaml::<HostConfig>(&host_dir, "host")?;

    // Load a template file if it exists and pre-seed some default templating values.
    let templating_vars = get_host_vars(&host_dir, hostname, &config)?;

    // Now we recursively read all files in the host directory
    // First, read the directory entries.
    let mut files = Directory::new(&host_dir);
    let entries = std::fs::read_dir(&host_dir)
        .map_err(|err| Error::IoPath(host_dir.clone(), "reading host dir", err))?;
    // Now got through all entries in this directory and recursively read them.
    for entry in entries {
        let entry =
            entry.map_err(|err| Error::IoPath(host_dir.clone(), "reading host dir entry", err))?;

        // Don't include the host or variable configuration file. It's already handled above
        if ["host.yml", "host.yaml", "vars.yml", "vars.yaml"]
            .contains(&entry.file_name().to_str().unwrap())
        {
            continue;
        }

        read_entry(
            &host_dir,
            Path::new(""),
            entry,
            &mut files,
            config.target_directory.clone(),
            &templating_vars,
        )?;
    }

    Ok(Host {
        config,
        variables: templating_vars,
        directory: files,
        groups: Vec::new(),
    })
}
