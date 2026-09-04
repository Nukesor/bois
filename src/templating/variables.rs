use std::{env::var, path::Path};

use anyhow::{Result, bail};
use nix::unistd::{Gid, Uid};
use serde_yaml::{Mapping, Value};

use crate::config::{helper::read_yaml, host::HostConfig};

/// Read the host's templating variables.
///
/// They either come from a `vars.yml` file in the host directory or from the `vars` field of the
/// host's config file. Specifying both is a conflict.
///
/// While at it, populate the variables with other useful variables that're exposed by defaults.
/// These include:
/// - The hostname itself
pub fn get_host_vars(host_dir: &Path, hostname: &str, config: &HostConfig) -> Result<Value> {
    // First up, check if there's a vars.yml file in the host directory.
    let vars_file_exists =
        host_dir.join("vars.yaml").exists() || host_dir.join("vars.yml").exists();

    // Variables must come from a single source.
    if vars_file_exists && config.vars.is_some() {
        bail!(
            "Found both a vars file in the host directory {host_dir:?} and a `vars` field \
             in the config file of host '{hostname}'. Define the templating variables in \
             only one of the two places. Aborting."
        );
    }

    // Get the variables from wherever they're defined.
    // If there're none, fall back to Null, which is treated as an empty map below.
    let value = if vars_file_exists {
        read_yaml::<Value>(host_dir, "vars")?
    } else if let Some(vars) = &config.vars {
        vars.clone()
    } else {
        Value::Null
    };

    // We expect vars to be a top level map, so yamls consisting of a single array will throw an
    // error. If no vars are specified or they're empty, start with an empty map.
    let mut variables = match value {
        Value::Mapping(map) => map,
        // An empty vars file or an empty `vars:` value deserializes to Null.
        Value::Null => Mapping::new(),
        _ => bail!("Expected map for variables. Got {value:#?}"),
    };

    // ----------- Default template variables -----------
    // The following block injects default variables that're always available during templating.

    // Insert the host variables
    variables.insert(
        serde_yaml::to_value("host").unwrap(),
        serde_yaml::to_value(hostname).unwrap(),
    );

    // Insert the list of all enabled traits for this host.
    variables.insert(
        serde_yaml::to_value("traits").unwrap(),
        serde_yaml::to_value(config.traits.clone()).unwrap(),
    );

    // Insert environment dependent variables, specifically which user currently executes boi.
    variables.insert(
        serde_yaml::to_value("USER_ID").unwrap(),
        serde_yaml::to_value(Uid::current().as_raw()).unwrap(),
    );
    variables.insert(
        serde_yaml::to_value("USER").unwrap(),
        serde_yaml::to_value(var("USER").unwrap_or_default()).unwrap(),
    );
    variables.insert(
        serde_yaml::to_value("GROUP_ID").unwrap(),
        serde_yaml::to_value(Gid::current().as_raw()).unwrap(),
    );

    Ok(Value::Mapping(variables))
}
