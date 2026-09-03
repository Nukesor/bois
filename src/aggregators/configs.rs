use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use serde_yaml::Value;

use crate::{
    config::{helper::read_yaml, host::HostConfig, traits::TraitConfig},
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

/// Read the host's config file and the `vars.yml` of the host directory.
///
/// A host without a directory consists of just a stand-alone config file at `hosts/<hostname>.yml`.
/// As soon as a directory exists, the config file is expected to be in that directory
/// `hosts/<hostname>/<hostname>.yml`.
pub fn read_host_config(root: &Path, hostname: &str) -> Result<Host> {
    let hosts_dir = root.join("hosts");
    let host_dir = hosts_dir.join(hostname);

    let Some(config_dir) = locate_config(&hosts_dir, hostname)? else {
        if host_dir.is_dir() {
            bail!(
                "Couldn't find a config file for host '{hostname}'. \
                 Expected it inside the host directory at {:?}. Aborting.",
                host_dir.join(format!("{hostname}.yml")),
            );
        }
        bail!(
            "Couldn't find host '{hostname}'. There's neither a config file at {:?} \
             nor a host directory at {host_dir:?}. Aborting.",
            hosts_dir.join(format!("{hostname}.yml")),
        );
    };
    let config = read_yaml::<HostConfig>(&config_dir, hostname)?;

    // Load a template file if it exists and pre-seed some default templating values.
    let variables = get_host_vars(&host_dir, hostname, &config)?;

    Ok(Host { config, variables })
}

/// Read a trait's config file.
///
/// A trait without a directory consists of just a stand-alone config file at
/// `traits/<traitname>.yml`.
/// If a directory exists, the (then optional) config file is expected to be in
/// that directory `hosts/<hostname>/<hostname>.yml`.
///
/// A missing config file results in a default config, as long as the trait's
/// directory exists.
/// [crate::aggregators::path::walk_source].
pub fn read_trait_config(root: &Path, name: &str) -> Result<TraitConfig> {
    let traits_dir = root.join("traits");
    let trait_dir = traits_dir.join(name);

    let config = match locate_config(&traits_dir, name)? {
        Some(config_dir) => read_yaml::<TraitConfig>(&config_dir, name)?,
        None => {
            if !trait_dir.is_dir() {
                bail!(
                    "Couldn't find trait '{name}'. There's neither a config file at {:?} \
                     nor a trait directory at {trait_dir:?}. Aborting.",
                    traits_dir.join(format!("{name}.yml")),
                );
            }
            TraitConfig::default()
        }
    };

    Ok(config)
}

/// Find the directory that holds a host's/trait's config file.
///
/// A host/trait either consists of just a stand-alone config file
/// (`hosts/<name>.yml`) or of its own directory, in which case the config file
/// lives inside that directory (`hosts/<name>/<name>.yml`). A stand-alone
/// config file next to a directory is an error.
///
/// `None` means no config file exists at the applicable location.
fn locate_config(base_dir: &Path, name: &str) -> Result<Option<PathBuf>> {
    let config_file_in = |directory: &Path, name: &str| {
        ["yml", "yaml"]
            .iter()
            .map(|extension| directory.join(format!("{name}.{extension}")))
            .find(|path| path.exists())
    };

    let own_dir = base_dir.join(name);
    let standalone = config_file_in(base_dir, name);

    if own_dir.is_dir() {
        if let Some(standalone) = standalone {
            bail!(
                "Found both a config file at {standalone:?} and a directory at {own_dir:?} \
                 for '{name}'. Either move the config file into the directory as {:?} \
                 or remove the directory. Aborting.",
                own_dir.join(format!("{name}.yml")),
            );
        }
        Ok(config_file_in(&own_dir, name).map(|_| own_dir))
    } else {
        Ok(standalone.map(|_| base_dir.to_path_buf()))
    }
}
