use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use log::info;

use crate::{state::PackageManager, system_state::SystemState};

/// Merge the host's and all traits' package sets into a single set per package
/// manager.
///
/// Duplicates across sources are legal but produce an info log.
/// Package *groups* (e.g. pacman's `base-devel`) are unrolled into their member packages, so the
/// desired set only contains real packages. This is to ensure all packages of a group exist (or
/// are cleaned up), as the set of packages included in a group may change at any point in time.
pub fn aggregate_packages(
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
