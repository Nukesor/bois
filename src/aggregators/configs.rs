use std::path::Path;

use anyhow::{Result, bail};
use serde_yaml::Value;

use crate::{
    config::{helper::read_yaml, host::HostConfig, traits::TraitConfig},
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
    /// All variables that're available for templating to the host files and all traits.
    pub variables: Value,
}

/// Read the `host.yml` and `vars.yml` of the host directory.
pub fn read_host_config(root: &Path, hostname: &str) -> Result<Host> {
    let host_dir = root.join("hosts").join(hostname);

    if !host_dir.exists() {
        bail!("Couldn't find config directory for this host at {host_dir:?}. Aborting.");
    }

    // Read the `host.yml` from the host directory.
    let config = read_yaml::<HostConfig>(&host_dir, "host")?;

    // Load a template file if it exists and pre-seed some default templating values.
    let variables = get_host_vars(&host_dir, hostname, &config)?;

    Ok(Host { config, variables })
}

/// Read the `trait.yml` of a trait directory.
///
/// A missing `trait.yml` is fine and results in a default config.
/// The trait directory's files are read later on via
/// [crate::aggregators::path::walk_source].
pub fn read_trait_config(root: &Path, name: &str) -> Result<TraitConfig> {
    let trait_dir = root.join("traits").join(name);

    if !trait_dir.exists() {
        bail!("Couldn't find config directory for trait '{name}' at {trait_dir:?}. Aborting.");
    }

    // Read the `trait.yml` from the trait directory.
    // Return a default config if the trait config doesn't exist.
    let config = match read_yaml::<TraitConfig>(&trait_dir, "trait") {
        Ok(config) => config,
        Err(error) => match error {
            Error::FileNotFound(_, _) => TraitConfig::default(),
            _ => bail!(error),
        },
    };

    Ok(config)
}
