//! Aggregation logic for a bois directory.
//!
//! This module reads the whole bois configuration for the current host and its enabled traits.
//! All that information is bundled into a single [State] struct.
//!
//! The resulting [State] struct is fully resolved, which means the following guarantees are upheld:
//! - Deduplication has taken place
//! - Any conflicts have been resolved. Unresolvable conflicts result in errors.
//! - Templating logic has been executed.
//!
//! As a result, the [State] is a valid representation of (managed part of) the system after the
//! deploy phase.

mod configs;
mod packages;
pub mod path;
mod services;

use anyhow::{Result, bail};
use configs::{read_host_config, read_trait_config};
use packages::aggregate_packages;
use path::{WalkContext, walk_source};
use services::aggregate_services;

use crate::{
    config::bois::Configuration,
    state::{State, path::Tree},
    system_state::SystemState,
};

/// Build the desired [State] for the current host from the bois directory.
pub fn aggregate_state(config: &Configuration, system_state: &mut SystemState) -> Result<State> {
    // Make sure the bois directory exists.
    let bois_dir = config.bois_dir.clone();
    if !bois_dir.exists() {
        bail!("Couldn't find bois directory at {bois_dir:?}. Aborting.");
    }

    // Read the config of the current host.
    let host = read_host_config(&bois_dir, &config.name)?;

    // Read the configs of all enabled traits.
    let mut traits = Vec::new();
    for trait_name in &host.config.traits {
        let trait_config = read_trait_config(&bois_dir, trait_name)?;
        traits.push((trait_name.clone(), trait_config));
    }

    // ---------- Services ----------
    // Merge the host's and all traits' service declarations.
    // Directory-declared services are merged afterewards when the source directories are walked.
    let mut services = aggregate_services(&host.config.services, &traits);

    // ---------- File tree ----------
    // Walk through the host directory and all trait directories in their
    // configured order.
    //
    // This step gathers all source files and directories for the deploy phase,
    // while resolving any conflicts, performing error handling and templating.
    let mut tree = Tree::new();

    walk_source(
        &WalkContext::for_host(config, &host.config, &config.name, &host.variables)?,
        &mut tree,
        &mut services,
    )?;

    for (name, trait_config) in &traits {
        walk_source(
            &WalkContext::for_trait(config, trait_config, name, &host.variables)?,
            &mut tree,
            &mut services,
        )?;
    }

    // ---------- Packages ----------
    let packages = aggregate_packages(&host.config.packages, &traits, system_state)?;

    Ok(State {
        configuration: config.clone(),
        path_tree: tree,
        packages,
        services,
    })
}
