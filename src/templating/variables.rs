use std::{env::var, path::Path};

use anyhow::{Result, bail};
use nix::unistd::{Gid, Uid};
use serde_yaml::{Mapping, Value};

use crate::{config::helper::read_yaml, state::host::HostConfig};

/// Read the `vars.yml` from a host directory if it exists.
///
/// While at it, populate the variables with other useful variables that're exposed by defaults.
/// These include:
/// - The hostname itself
pub fn get_host_vars(host_dir: &Path, hostname: &str, config: &HostConfig) -> Result<Value> {
    // First up, read the vars.yml file and convert it into a [serde_yaml::Value].
    let vars_file_exists =
        host_dir.join("vars.yaml").exists() || host_dir.join("vars.yml").exists();

    // We expect vars to a top level map, so yamls consisting of a single array will throw an
    // error. If no file is found, return an empty map.
    let mut variables = if vars_file_exists {
        let value = read_yaml::<Value>(host_dir, "vars")?;
        match value {
            Value::Mapping(map) => map,
            _ => bail!("Expected map for variables. Got {value:#?}"),
        }
    } else {
        Mapping::new()
    };

    // ----------- Default template variables -----------
    // The following block injects default variables that're always available during templating.

    // Insert the host variables
    variables.insert(
        serde_yaml::to_value("host").unwrap(),
        serde_yaml::to_value(hostname).unwrap(),
    );

    // Insert the list of all enabled groups for this host.
    variables.insert(
        serde_yaml::to_value("boi_groups").unwrap(),
        serde_yaml::to_value(config.groups.clone()).unwrap(),
    );

    // Insert environment dependant variables, specifically which user currently executes boi.
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
