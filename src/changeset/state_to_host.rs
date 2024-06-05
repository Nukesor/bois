use anyhow::Result;

use crate::{
    state::{group::Group, host::Host, State},
    system_state::SystemState,
};

use super::{Change, ChangeSet, PackageOperation};

pub fn create_changeset(
    state: &State,
    system_state: &mut SystemState,
) -> Result<Option<ChangeSet>> {
    let mut changeset = Vec::new();

    // Create changeset for missing packages.
    let package_changeset = handle_packages(state, system_state)?;
    changeset.extend(package_changeset);

    // Create changeset for files and system services on host config.
    let host_changeset = handle_host(&state.host, system_state)?;
    changeset.extend(host_changeset);

    // Create changeset for files and system services on group configs.
    for group in state.host.groups.iter() {
        let group_changset = handle_group(group, system_state)?;
        changeset.extend(group_changset);
    }

    if changeset.is_empty() {
        Ok(None)
    } else {
        Ok(Some(changeset))
    }
}

/// Detect any packages that're missing on the current config and queue them for installation.
fn handle_packages(state: &State, system_state: &mut SystemState) -> Result<ChangeSet> {
    let mut changeset = Vec::new();

    // Compare all desired packages in the top-level config with the currently installed one's.
    for (manager, packages) in state.packages.iter() {
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

/// Create the changeset that's needed to reach the desired state of the [HostConfig] from the
/// current system's state.
fn handle_host(_host: &Host, _system_state: &mut SystemState) -> Result<ChangeSet> {
    let changeset = Vec::new();

    Ok(changeset)
}

/// Create the changeset that's needed to reach the desired state of a given [GroupConfig] from the
/// current system's state.
fn handle_group(_group: &Group, _system_state: &mut SystemState) -> Result<ChangeSet> {
    let changeset = Vec::new();

    Ok(changeset)
}
