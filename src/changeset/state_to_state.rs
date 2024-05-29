use anyhow::Result;
use log::info;

use crate::{state::State, system_state::SystemState};

use super::{compiled_state::CompiledState, ChangeSet};

/// Compare a new desired State with a previously deployed state.
/// This is used to determine any necessary **cleanup** operations, in case the previous deployment
/// enabled services or contained files, packages that're no longer desired.
///
/// We do this by creating a "compiled state", which represents the final state that will be
/// deployed on the system.
/// We're not interested in where these files come from but rather only wether those files need to
/// be removed, which is why this simplified and easier to handle representation is sufficient.
pub fn create_changeset(
    system_state: &mut SystemState,
    old_state: &State,
    new_state: &State,
) -> Result<Option<ChangeSet>> {
    let mut changeset = ChangeSet::new();

    let old_compiled_state = CompiledState::from_state(old_state);
    let new_compiled_state = CompiledState::from_state(new_state);

    handle_packages(
        system_state,
        &mut changeset,
        &old_compiled_state,
        &new_compiled_state,
    )?;

    if changeset.is_empty() {
        Ok(None)
    } else {
        Ok(Some(changeset))
    }
}

/// Check all package managers on the old system and their respective packages.
/// Check for each package whether it existed on the old system.
/// If not, queue a change to remove it.
pub fn handle_packages(
    system_state: &mut SystemState,
    changeset: &mut ChangeSet,
    old_state: &CompiledState,
    new_state: &CompiledState,
) -> Result<()> {
    for (manager, old_packages) in old_state.deployed_packages.iter() {
        // First up, check if there're any packages for this package manager at all.
        // If we cannot find a package manager, remove all packages that were deployed for it.
        let Some(new_packages) = new_state.deployed_packages.get(&manager) else {
            for package in old_packages {
                changeset.push(super::Change::PackageChange(
                    super::PackageOperation::Remove {
                        manager: *manager,
                        name: package.clone(),
                    },
                ))
            }
            continue;
        };

        // Now we compare the old and the new desired state, queuing removal of packages if they're
        // no longer explicitly installed.
        let installed_packages = system_state.installed_packages(*manager)?;
        for package in new_packages {
            if new_packages.contains(package) {
                continue;
            }

            // Ignore it if it has already been removed from the target system.
            if !installed_packages.contains(package) {
                info!("Package '{package}'t o be removed does no longer exist on system.");
                continue;
            }

            // Package wasn't found in new state, queue for removal.
            changeset.push(super::Change::PackageChange(
                super::PackageOperation::Remove {
                    manager: *manager,
                    name: package.clone(),
                },
            ))
        }
    }

    Ok(())
}
