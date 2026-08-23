//! Comparison 3: desired state -> last-deployed state.
//!
//! This module determines all parts of a previous run that're no longer
//! in the desired state.
//!
//! Such system state that must be cleaned up consists of:
//! - Files/directories that were removed from (or moved within) the bois directory.
//! - Packages that were dropped from the config.
//! - Services that were dropped from the config. Those are stopped and disabled.
//!
//! On top of that, any detected cleanup operation is validated against the system
//! and should not be reported if no longer necessary.

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;

use crate::{
    changeset::{
        Changeset,
        DirectoryOperation,
        FileOperation,
        PackageUninstall,
        PathOperation,
        ServiceDisable,
    },
    state::{
        State,
        path::{DirectoryBacking, Node, Tree},
    },
    system_state::{SystemState, entry::ActualEntry},
};

/// Compute the state that remains from a previous run after the cleanup phase
/// has been executed.
///
/// This state is then persisted between the cleanup and deploy phases of a run.
/// If the deploy phase aborts halfway the next run's drift detection would otherwise
/// blame the executed cleanup deletions on the user as drift on your system.
pub fn post_cleanup_state(previous: &State, cleanup: &Changeset) -> State {
    let mut state = previous.clone();

    // The cleanup operations are ordered leaf-to-root, so children are
    // removed before their parent directories.
    for operation in &cleanup.path_cleanup {
        state.path_tree.remove(operation.path());
    }

    for uninstall in &cleanup.package_uninstalls {
        if let Some(packages) = state.packages.get_mut(&uninstall.manager) {
            packages.remove(&uninstall.name);
        }
    }

    for disable in &cleanup.service_disables {
        if let Some(services) = state.services.get_mut(&disable.manager) {
            services.retain(|service| service.name != disable.name);
        }
    }

    state
}

/// Compare the previously deployed state with the desired state and
/// create the changeset of all necessary cleanup operations.
///
/// This comparison is "filetype-sensitive" This means that a path that's a file in the previous
/// state and a directory in the desired state (or vice versa) counts as removed.
pub fn cleanup_changeset(
    previous: &State,
    desired: &State,
    system_state: &mut SystemState,
) -> Result<Changeset> {
    let mut changeset = Changeset::new();

    handle_paths(&previous.path_tree, &desired.path_tree, &mut changeset)?;
    handle_packages(previous, desired, system_state, &mut changeset)?;
    handle_services(previous, desired, system_state, &mut changeset)?;

    Ok(changeset)
}

/// Queue deletions for all previously deployed paths that're absent from the
/// desired state.
///
/// The previous tree is walked in leaf-to-root order, so files and subdirectories are deleted
/// before their parent directories.
fn handle_paths(previous: &Tree, desired: &Tree, changeset: &mut Changeset) -> Result<()> {
    // Lookup of the desired state's paths with all parent symlinks resolved.
    let resolved_desired_paths: HashMap<PathBuf, bool> = desired
        .flatten()
        .into_iter()
        .filter_map(|(path, node)| {
            let resolved = path.canonicalize().ok()?;
            Some((resolved, matches!(node, Node::Directory(_))))
        })
        .collect();

    for (path, node) in previous.flatten().into_iter().rev() {
        // Anything that's still present in the desired state should not be cleaned up.
        // So we check if the desired state still contains this exact path with the same filetype.
        let still_present = match (desired.get(&path), node) {
            (Some(Node::File(_)), Node::File(_))
            | (Some(Node::Directory(_)), Node::Directory(_)) => true,
            // Either missing, or present with a different filetype.
            _ => false,
        };
        if still_present {
            continue;
        }

        // The path might have been renamed to a path that resolves to the same physical location
        // through symlinks. Cleaning up the previous path would then delete the file although it
        // should remain untouched. Because of this, the following logic checks for such
        // symlink "renames".
        //
        // For a conflict to happen, the previous path must exist and resolve.
        if path.exists()
            && let Ok(resolved) = path.canonicalize()
        {
            // And the desired state must have a path with the same type that resolves to the same
            // value.
            if let Some(is_desired_dir) = resolved_desired_paths.get(&resolved)
                && *is_desired_dir == matches!(node, Node::Directory(_))
            {
                continue;
            }
        }

        match node {
            Node::File(_) => {
                if let ActualEntry::File { .. } = ActualEntry::read(&path)? {
                    changeset
                        .path_cleanup
                        .push(PathOperation::File(FileOperation::Cleanup { path }));
                }
            }
            Node::Directory(dir) => {
                // Only cleanup directories that were explicitly handled by bois
                // and where cleanup is explicitly enabled.
                let DirectoryBacking::Backed(meta) = &dir.backing else {
                    continue;
                };
                if !meta.cleanup {
                    continue;
                }

                if let ActualEntry::Directory { .. } = ActualEntry::read(&path)? {
                    changeset.path_cleanup.push(PathOperation::Directory(
                        DirectoryOperation::Cleanup { path },
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Check for any packages that were previously deployed but are no longer
/// part of the desired state and queue them for removal.
fn handle_packages(
    previous: &State,
    desired: &State,
    system_state: &mut SystemState,
    changeset: &mut Changeset,
) -> Result<()> {
    for (manager, previous_packages) in previous.packages.iter() {
        let desired_packages = desired.packages.get(manager);

        // Uninstalling only makes sense for packages that're currently
        // installed as **explicit** packages. If a package was demoted to a
        // dependency or is already gone, there's nothing to do.
        // TODO: Check out the super::deploy::handle_packages function for why we
        // need a better dependency solver.
        let explicit_packages = system_state.explicit_packages(*manager)?;

        for previous_package in previous_packages {
            let still_desired =
                desired_packages.is_some_and(|packages| packages.contains(previous_package));
            if still_desired {
                continue;
            }

            // TODO: Check out the super::deploy::handle_packages function for why we
            // need a better dependency solver.
            if explicit_packages.contains(previous_package) {
                changeset.package_uninstalls.push(PackageUninstall {
                    manager: *manager,
                    name: previous_package.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Check for any services that were previously deployed but are no longer
/// part of the desired state and queue them to be stopped + disabled.
fn handle_services(
    previous: &State,
    desired: &State,
    system_state: &mut SystemState,
    changeset: &mut Changeset,
) -> Result<()> {
    for (manager, previous_services) in previous.services.iter() {
        let desired_services = desired.services.get(manager);

        for previous_service in previous_services {
            let still_desired = desired_services.is_some_and(|services| {
                services
                    .iter()
                    .any(|desired| desired.name == previous_service.name)
            });
            if still_desired {
                continue;
            }

            // Only queue services that're actually still enabled or running. Anything else, like a
            // unit that was manually disabled or vanished, needs no cleanup.
            if system_state.service_enabled(*manager, &previous_service.name)?
                || system_state.service_active(*manager, &previous_service.name)?
            {
                changeset.service_disables.push(ServiceDisable {
                    manager: *manager,
                    name: previous_service.name.clone(),
                });
            }
        }
    }

    Ok(())
}
