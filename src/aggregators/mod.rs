//! Aggregation logic for a bois source directory.
//!
//! This module reads the whole bois configuration for the current host and its enabled traits.
//! All that information is bundled into a single [State] struct.
//!
//! The resulting [State] struct is fully resolved, which means the following guarantees are upheld:
//! - Deduplication has taken place
//! - Any conflicts have been resolved. Unresolvable conflicts result in errors.
//! - Templating logic has been executed.
//!
//! As a result, the [State] is a valid representation of (managed part of) the system after
//! deployment.

mod configs;
pub mod path;

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, bail};
use configs::{read_host_config, read_trait_config};
use log::info;
use path::{WalkContext, walk_source};

use crate::{
    config::bois::Configuration,
    state::{PackageManager, State, path::Tree},
    system_state::SystemState,
};

/// Build the desired [State] for the current machine from the bois
/// configuration directory.
pub fn aggregate_state(config: &Configuration, system_state: &mut SystemState) -> Result<State> {
    // Make sure the bois source directory exists.
    let bois_dir = config.bois_dir.clone();
    if !bois_dir.exists() {
        bail!("Couldn't find bois config directory at {bois_dir:?}. Aborting.");
    }

    // Read the host config for this machine.
    let host = read_host_config(&bois_dir, &config.name)?;

    // Read the configs of all enabled traits.
    let mut traits = Vec::new();
    for trait_name in &host.config.traits {
        let trait_config = read_trait_config(&bois_dir, trait_name)?;
        traits.push((trait_name.clone(), trait_config));
    }

    // ---------- File tree ----------
    // Walk through the host directory and all trait directories in their
    // configured order.
    //
    // This step gathers all source files and directories for deployment,
    // while resolving any conflicts, performing erro rhandling and templating.
    let mut tree = Tree::new();

    walk_source(
        &WalkContext::for_host(config, &host.config, &config.name, &host.variables)?,
        &mut tree,
    )?;

    for (name, trait_config) in &traits {
        walk_source(
            &WalkContext::for_trait(config, trait_config, name, &host.variables)?,
            &mut tree,
        )?;
    }

    // ---------- Packages ----------
    let packages = aggregate_packages(&host.config.packages, &traits, system_state)?;

    Ok(State {
        configuration: config.clone(),
        path_tree: tree,
        packages,
    })
}

/// Merge the host's and all traits' package sets into a single set per package
/// manager.
///
/// Duplicates across sources are legal but produce an info log.
/// Package *groups* (e.g. pacman's `base-devel`) are unrolled into their member packages, so the
/// desired set only contains real packages. This is to ensure all packages of a group exist (or
/// are cleaned up), as the set of packages included in a group may change at any point in time.
fn aggregate_packages(
    host_packages: &BTreeMap<PackageManager, BTreeSet<String>>,
    traits: &[(String, crate::config::traits::TraitConfig)],
    system_state: &mut SystemState,
) -> Result<BTreeMap<PackageManager, BTreeSet<String>>> {
    let mut packages: BTreeMap<PackageManager, BTreeSet<String>> = BTreeMap::new();

    let merge = |source: &str,
                 new: &BTreeMap<PackageManager, BTreeSet<String>>,
                 known: &mut BTreeMap<PackageManager, BTreeSet<String>>| {
        for (manager, new_packages) in new {
            let known_packages = known.entry(*manager).or_default();
            for duplicate in new_packages.intersection(known_packages) {
                info!("Found duplicate package '{duplicate}' in {source}");
            }
            known_packages.extend(new_packages.iter().cloned());
        }
    };

    merge("host.yml", host_packages, &mut packages);
    for (name, trait_config) in traits {
        merge(
            &format!("trait.yml of trait '{name}'"),
            &trait_config.packages,
            &mut packages,
        );
    }

    // If any of the configured "packages" is actually a package group known to
    // the system, unroll it into its member packages and remove the group name.
    for (manager, packages) in packages.iter_mut() {
        let groups_on_system = system_state.package_groups(*manager)?.clone();

        let detected_groups: BTreeSet<String> =
            packages.intersection(&groups_on_system).cloned().collect();

        for group in &detected_groups {
            let group_packages = system_state.packages_for_group(*manager, group)?;
            packages.extend(group_packages);
        }
        packages.retain(|name| !detected_groups.contains(name));
    }

    Ok(packages)
}
