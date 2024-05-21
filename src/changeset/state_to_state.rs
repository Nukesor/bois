use anyhow::Result;

use crate::{
    state::{group::Group, host::Host, State},
    system_state::SystemState,
};

use super::{Change, ChangeSet, PackageOperation};

/// Compare a new desired State with a previously deployed state.
/// This is used to determine any necessary **cleanup** operations, in case the previous deployment
/// enabled services or contained files, packages that're no longer desired.
pub fn create_changeset(old_state: &State, new_state: &State) -> Result<ChangeSet> {
    let mut changeset = Vec::new();

    let host_changset = handle_host(&old_state.host, &new_state.host)?;
    changeset.extend(host_changset);

    for old_group in old_state.host.groups.iter() {
        let group_changset = handle_group(&old_group, new_group)?;
        changeset.extend(group_changset);
    }

    Ok(changeset)
}

/// Create the changeset that's needed to reach the desired state of the [HostConfig] from the
/// current system's state.
fn handle_host(old_state: &Host, new_state: &Host) -> Result<ChangeSet> {
    let mut changeset = Vec::new();

    // Compare all desired packages in the top-level config with the currently installed one's.
    // TODO: How to handle package-groups? E.g. xorg-apps
    for (manager, packages) in host.config.packages.iter() {
        let installed_packages = system_state.installed_packages(*manager)?;
        for package in packages {
            // If a package is not found, schedule it to be installed.
            if !installed_packages.contains(package) {
                changeset.push(Change::PackageChange(PackageOperation::Add {
                    manager: *manager,
                    name: package.clone(),
                }))
            }
        }
    }

    Ok(changeset)
}

/// Create the changeset that's needed to reach the desired state of a given [GroupConfig] from the
/// current system's state.
fn handle_group(group: &Group, system_state: &mut SystemState) -> Result<ChangeSet> {
    let mut changeset = Vec::new();

    // Compare all desired packages in the top-level config with the currently installed one's.
    for (manager, packages) in group.config.packages.iter() {
        let installed_packages = system_state.installed_packages(*manager)?;
        for package in packages {
            // If a package is not found, schedule it to be installed.
            if !installed_packages.contains(package) {
                changeset.push(Change::PackageChange(PackageOperation::Add {
                    manager: *manager,
                    name: package.clone(),
                }))
            }
        }
    }

    Ok(changeset)
}
