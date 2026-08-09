//! Comparison 3: desired state -> last-deployed state.
//!
//! This module determines all parts of a previous deployment that're no longer
//! in the desired state.
//!
//! Such system state that must be cleaned up consists of:
//! - Files/directories that were removed from (or moved within) the bois dir.
//! - Packages that were dropped from the config.
//!
//! On top of that, any detected cleanup operation is validated against the live system
//! and should not be reported if no longer necessary.

use std::path::Path;

use anyhow::Result;

use crate::{
    changeset::{
        Changeset,
        DirectoryOperation,
        FileOperation,
        PackageUninstall,
        PathOperation,
        system::{LiveEntry, read_live_entry},
    },
    state::{
        State,
        path::{DirectoryBacking, Node},
    },
    system_state::SystemState,
};

/// Compare the previously deployed state with the new desired state and
/// create the changeset of all necessary cleanup operations.
///
/// This comparison is "filetype-sensitive" This means that a path that's a file in the old state
/// and a directory in the new state (or vice versa) counts as removed.
pub fn cleanup_changeset(
    old: &State,
    new: &State,
    system_state: &mut SystemState,
) -> Result<Changeset> {
    let mut changeset = Changeset::new();

    handle_paths(old, new, &mut changeset)?;
    handle_packages(old, new, system_state, &mut changeset)?;

    Ok(changeset)
}

/// Queue deletions for all previously deployed paths that're absent from the
/// new state.
///
/// The old tree is walked in leaf-to-root order, so files and subdirectories are deleted before
/// their parent directories.
fn handle_paths(old: &State, new: &State, changeset: &mut Changeset) -> Result<()> {
    for (path, node) in old.path_tree.flatten().into_iter().rev() {
        // Anything that's still present in the new state should not be cleaned up.
        if still_present(new, &path, node) {
            continue;
        }

        match node {
            Node::File(_) => {
                if let LiveEntry::File { .. } = read_live_entry(&path)? {
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

                if let LiveEntry::Directory { .. } = read_live_entry(&path)? {
                    changeset.path_cleanup.push(PathOperation::Directory(
                        DirectoryOperation::Cleanup { path },
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Whether the new state still contains this exact path with the same filetype.
fn still_present(new: &State, path: &Path, old_node: &Node) -> bool {
    match (new.path_tree.get(path), old_node) {
        (Some(Node::File(_)), Node::File(_)) => true,
        (Some(Node::Directory(_)), Node::Directory(_)) => true,
        // Either missing, or present with a different filetype.
        _ => false,
    }
}

/// Compute the state that remains from a previous deployment after the cleanup phase
/// has been executed.
///
/// This state is then persisted between the cleanup and deploy phases of a run.
/// If the deploy phase aborts halfway the next run's drift detection would otherwise
/// blame the executed cleanup deletions on the user as untracked changes on your system.
pub fn post_cleanup_state(old: &State, cleanup: &Changeset) -> State {
    let mut state = old.clone();

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

    state
}

/// Check for any packages that were previously deployed but are no longer
/// part of the desired state and queue them for removal.
fn handle_packages(
    old: &State,
    new: &State,
    system_state: &mut SystemState,
    changeset: &mut Changeset,
) -> Result<()> {
    for (manager, old_packages) in old.packages.iter() {
        let new_packages = new.packages.get(manager);

        // Uninstalling only makes sense for packages that're currently
        // installed as **explicit** packages. If a package was demoted to a
        // dependency or is already gone, there's nothing to do.
        // TODO: Check out the super::deploy::handle_packages function for why we
        // need a better dependency solver.
        let explicit_packages = system_state.explicit_packages(*manager)?;

        for old_package in old_packages {
            let still_desired = new_packages.is_some_and(|packages| packages.contains(old_package));
            if still_desired {
                continue;
            }

            // TODO: Check out the super::deploy::handle_packages function for why we
            // need a better dependency solver.
            if explicit_packages.contains(old_package) {
                changeset.package_uninstalls.push(PackageUninstall {
                    manager: *manager,
                    name: old_package.clone(),
                });
            }
        }
    }

    Ok(())
}
