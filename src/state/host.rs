use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::{bail, Result};
use serde_derive::{Deserialize, Serialize};

use crate::{handlers::packages::PackageManager, helper::read_yaml};

use super::{directory::*, group::Group};

/// A Host is related to a
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Host {
    /// The top-level configuration file for this host.
    pub config: HostConfig,
    /// The content of this group's directory.
    pub files: Directory,
    /// Will contain all groups that have been specified as dependencies.
    pub groups: Vec<Group>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostConfig {
    /// Default that should be applied to all files.
    #[serde(default)]
    pub file_defaults: HostDefaults,
    /// All variables that're available for templating to the host files and all groups.
    #[serde(default)]
    pub global_variables: HashMap<String, String>,
    /// All variables that're only available for templating files in this directory.
    #[serde(default)]
    pub local_variables: HashMap<String, String>,
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

    // Recursively read all files in directory
    let files = Directory::new(&host_dir);
    //let entries = std::fs::read_dir(&host_dir)
    //    .map_err(|err| Error::IoPathError(host_dir.clone(), "reading", err))?;
    //// Go through all entries in this directory
    //for entry in entries {
    //    let entry =
    //        entry.map_err(|err| Error::IoPathError(host_dir.clone(), "reading entry", err))?;

    //    read_file(root, &Path::new(""), entry, &mut files)?;
    //}

    Ok(Host {
        config,
        files,
        groups: Vec::new(),
    })
}
