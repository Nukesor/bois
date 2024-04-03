use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context, Result};
use serde_derive::{Deserialize, Serialize};

use crate::{error::Error, helper::read_yaml};

use super::{directory::*, file::read_file};

/// A Host is related to a
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Host {
    /// The top-level configuration file for this group/host.
    pub config: HostConfig,
    /// The content of this group's directory.
    pub files: Directory,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HostConfig {
    /// Default that should be applied to all files.
    pub defaults: HostDefaults,
    /// All variables that're available to all groups during templating.
    pub global_variables: HashMap<String, String>,
    /// All variables that're available during templating for this group.
    pub variables: HashMap<String, String>,
    /// Other groups that're required by this group.
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HostDefaults {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub file_permissions: Option<u32>,
    pub directory_permissions: Option<u32>,
}

pub fn read_host(root: &Path, name: &str) -> Result<Host> {
    let host_dir = root.join(name);

    // Read the `host.yml` from the host directory.
    let Some(host_config) = read_yaml::<HostConfig>(&host_dir, "host") else {
        bail!("Couldn't find host.yml in {host_dir:?}");
    };
    // Handle any deserialization issues of the host.yml
    let host_config = match host_config {
        Ok(host_config) => host_config,
        Err(err) => return Err(err).context("Failed to read host.yml in {host_dir:?}"),
    };

    // Recursively read all files in directory
    let mut files = Directory::new(&host_dir);
    let entries = std::fs::read_dir(&host_dir)
        .map_err(|err| Error::IoPathError(host_dir.clone(), "reading", err))?;
    // Go through all entries in this directory
    for entry in entries {
        let entry =
            entry.map_err(|err| Error::IoPathError(host_dir.clone(), "reading entry", err))?;

        read_file(root, &Path::new(""), entry, &mut files)?;
    }

    Ok(Host {
        config: host_config,
        files,
        ..Default::default()
    })
}
