use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::{bail, Result};
use serde_derive::{Deserialize, Serialize};

use crate::{error::Error, handlers::packages::PackageManager, helper::read_yaml};

use super::{directory::*, file::read_entry, group::Group};

/// A Host is related to a
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Host {
    /// The top-level configuration file for this host.
    pub config: HostConfig,
    /// All variables that're available for templating to the host files and all groups.
    #[serde(default)]
    pub variables: Option<serde_yaml::Value>,
    /// The content of this group's directory.
    pub directory: Directory,
    /// Will contain all groups that have been specified as dependencies.
    pub groups: Vec<Group>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostConfig {
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
    pub file_permissions: Option<u32>,
    pub directory_permissions: Option<u32>,
}

pub fn read_host(root: &Path, name: &str) -> Result<Host> {
    let host_dir = root.join("hosts").join(name);

    if !host_dir.exists() {
        eprintln!("Couldn't find config directory for this machine at {host_dir:?}. Aborting.");
        bail!("Couldn't find host config directory.");
    }

    // Read the `host.yml` from the host directory.
    let config = read_yaml::<HostConfig>(&host_dir, "host")?;

    // Read the `vars.yml` from the host directory if it exists.
    let variables = if host_dir.join("vars.yaml").exists() {
        Some(read_yaml::<serde_yaml::Value>(&host_dir, "vars")?)
    } else {
        None
    };

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
            None,
            &variables,
        )?;
    }

    Ok(Host {
        config,
        variables,
        directory: files,
        groups: Vec::new(),
    })
}
