use std::path::Path;

use anyhow::{Result, bail};
use serde_yaml::Value;

use crate::{
    config::{group::GroupConfig, helper::read_yaml, host::HostConfig},
    error::Error,
    templating::variables::get_host_vars,
};

/// The parsed top-level configuration of a host directory.
/// This is aggregation-internal: its contents are resolved into the
/// [crate::state::State] and not kept around afterwards.
#[derive(Clone, Debug, Default)]
pub struct Host {
    /// The top-level configuration file for this host.
    pub config: HostConfig,
    /// All variables that're available for templating to the host files and all groups.
    pub variables: Value,
}

/// Read the `host.yml` and `vars.yml` of the host directory.
pub fn read_host_config(root: &Path, hostname: &str) -> Result<Host> {
    let host_dir = root.join("hosts").join(hostname);

    if !host_dir.exists() {
        bail!("Couldn't find config directory for this machine at {host_dir:?}. Aborting.");
    }

    // Read the `host.yml` from the host directory.
    let config = read_yaml::<HostConfig>(&host_dir, "host")?;

    // Load a template file if it exists and pre-seed some default templating values.
    let variables = get_host_vars(&host_dir, hostname, &config)?;

    Ok(Host { config, variables })
}

/// Read the `group.yml` of a group directory.
///
/// A missing `group.yml` is fine and results in a default config.
/// The group directory's files are read later on via
/// [crate::aggregators::path::walk_source].
pub fn read_group_config(root: &Path, name: &str) -> Result<GroupConfig> {
    let group_dir = root.join("groups").join(name);

    if !group_dir.exists() {
        bail!("Couldn't find config directory for group '{name}' at {group_dir:?}. Aborting.");
    }

    // Read the `group.yml` from the group directory.
    // Return a default config if the group config doesn't exist.
    let config = match read_yaml::<GroupConfig>(&group_dir, "group") {
        Ok(config) => config,
        Err(error) => match error {
            Error::FileNotFound(_, _) => GroupConfig::default(),
            _ => bail!(error),
        },
    };

    Ok(config)
}
