//! This module contains logic to compare the current state with the state of the last deploy.
//! This allows us to detect any untracked changes on the system that have been done since the
//! last deploy.
//! We can then inform the user about these changes, so they aren't unintentionally overwritten.
use anyhow::Result;

use crate::{
    state::{group::Group, host::Host, State},
    system_state::SystemState,
};

use super::{Change, ChangeSet, PackageOperation};

pub fn create_changeset(
    system_state: &mut SystemState,
    old_state: &State,
    new_state: &State,
) -> Result<Option<ChangeSet>> {
    let mut changeset = Vec::new();

    // Create changeset for missing packages.
    let package_changeset = handle_packages(system_state, old_state, new_state)?;
    changeset.extend(package_changeset);

    // Create changeset for files and system services on host config.
    let host_changeset = handle_host(&old_state.host, system_state)?;
    changeset.extend(host_changeset);

    // Create changeset for files and system services on group configs.
    for group in old_state.host.groups.iter() {
        let group_changset = handle_group(group, system_state)?;
        changeset.extend(group_changset);
    }

    if changeset.is_empty() {
        Ok(None)
    } else {
        Ok(Some(changeset))
    }
}

/// Check for any packages that were previously on the system but have been removed.
///
/// Create a PackageChange for removal, so the user can understand the changes.
/// Make sure they haven't been removed from the new desired state before though.
/// We don't want to show removed packages that aren't desired.
fn handle_packages(
    system_state: &mut SystemState,
    old_state: &State,
    new_state: &State,
) -> Result<ChangeSet> {
    let mut changeset = Vec::new();

    // Compare all desired packages in the old config with the currently installed ones.
    for (manager, old_packages) in old_state.packages.iter() {
        let installed_packages = system_state.installed_packages(*manager)?;
        for old_package in old_packages {
            if !installed_packages.contains(old_package) {
                // Check if the removed package is supposed to be there.
                // If it isn't, just ignore this.
                if let Some(new_packages) = new_state.packages.get(manager) {
                    if !new_packages.contains(old_package) {
                        continue;
                    }
                }

                // The package has actually been removed, but is actually still desired.
                changeset.push(Change::PackageChange(PackageOperation::Remove {
                    manager: *manager,
                    name: old_package.clone(),
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
