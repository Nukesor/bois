use crate::state::State;

use super::{compiled_state::CompiledState, ChangeSet};

/// Compare a new desired State with a previously deployed state.
/// This is used to determine any necessary **cleanup** operations, in case the previous deployment
/// enabled services or contained files, packages that're no longer desired.
///
/// We do this by creating a "compiled state", which represents the final state that will be
/// deployed on the system.
/// We're not interested in where these files come from but rather only wether those files need to
/// be removed, which is why this simplified and easier to handle representation is sufficient.
pub fn create_changeset(old_state: &State, new_state: &State) -> Option<ChangeSet> {
    let mut changeset = ChangeSet::new();

    let old_compiled_state = CompiledState::from_state(old_state);
    let new_compiled_state = CompiledState::from_state(new_state);

    handle_packages(&mut changeset, &old_compiled_state, &new_compiled_state);

    if changeset.is_empty() {
        None
    } else {
        Some(changeset)
    }
}

/// Check for any packages that exist on the old state (the currently deployed system)
pub fn handle_packages(
    changeset: &mut ChangeSet,
    old_state: &CompiledState,
    new_state: &CompiledState,
) {
    // Iterate over all package managers on the old system and their respective packages.
    // Check for each package whether it existed on the old system. If not, queue a change to remove it.
    for (manager, old_packages) in old_state.deployed_packages.iter() {
        let Some(new_packages) = new_state.deployed_packages.get(&manager) else {
            // If we cannot find a package manager, remove all packages that were deployed for it.
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

        for package in old_packages {
            if new_packages.contains(package) {
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
}
