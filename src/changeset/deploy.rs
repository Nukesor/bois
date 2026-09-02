//! Comparison 1: desired state -> actual state.
//!
//! [deploy_changeset] determines what has to happen so the system reaches the desired state.
//!
//! There're a few rules regarding path modifications:
//! - The case that a different filetype is detected at the target location than desired. For
//!   example the user has a config that should be deployed to `~/.config/my_project`, but there's
//!   already a directory at that location. In those cases, the existing path is simply deleted,
//!   before the new file is created.

use anyhow::Result;

use crate::{
    changeset::{
        Changeset,
        DirectoryOperation,
        FileOperation,
        FileType,
        Modified,
        PackageInstall,
        PathOperation,
        ServiceEnable,
    },
    constants::{CURRENT_GROUP, CURRENT_USER},
    state::{
        State,
        path::{DirectoryPermissions, DirectoryState, FileState, Node, Tree},
    },
    system_state::{
        SystemState,
        entry::{ActualEntry, points_to_directory, read_actual_content},
    },
};

/// Create the changeset that transforms the system into the desired state.
///
/// - The target [`State::path_tree`] is checked against the filesystem
/// - Packages and services are checked against the cached system state.
pub fn deploy_changeset(desired: &State, system_state: &mut SystemState) -> Result<Changeset> {
    let mut changeset = Changeset::new();

    handle_paths(&desired.path_tree, &mut changeset)?;
    handle_packages(desired, system_state, &mut changeset)?;
    handle_services(desired, system_state, &mut changeset)?;

    Ok(changeset)
}

/// Recursively walk the target path tree and compare each node with the filesystem.
///
/// This relies on [Tree::flatten] returning a pre-order walk:
/// - Parents come before their children
/// - A directory's subtree is contiguous.
fn handle_paths(desired: &Tree, changeset: &mut Changeset) -> Result<()> {
    // The root of the last directory subtree that will be created from scratch during
    // execution. Either nothing exists at that path yet, or a conflicting actual entry (e.g. a
    // symlink) gets deleted and replaced with a new directory.
    //
    // Either way, nothing can exist below that path once the directory is created, so all
    // descendants are compared as missing.
    let mut new_dir: Option<std::path::PathBuf> = None;

    for (path, node) in desired.flatten() {
        let inside_new_dir = new_dir.as_ref().is_some_and(|root| path.starts_with(root));

        let actual = if inside_new_dir {
            ActualEntry::Missing
        } else {
            ActualEntry::read(&path)?
        };

        match node {
            Node::File(file) => handle_file(&path, file, &actual, changeset)?,
            Node::Directory(dir) => match handle_directory(&path, dir, &actual, changeset)? {
                DirectoryOutcome::Kept => (),
                DirectoryOutcome::Created | DirectoryOutcome::Replaced => {
                    if !inside_new_dir {
                        // Set `new_dir` in case the directory is newly created, so that all
                        // descendants are marked as missing.
                        new_dir = Some(path);
                    }
                }
            },
        }
    }

    Ok(())
}

fn handle_file(
    path: &std::path::Path,
    file: &FileState,
    actual: &ActualEntry,
    changeset: &mut Changeset,
) -> Result<()> {
    let create = || {
        PathOperation::File(FileOperation::Create {
            path: path.to_path_buf(),
            content: file.content.clone(),
            mode: file.mode,
            owner: file.owner.clone(),
            group: file.group.clone(),
        })
    };

    match actual {
        ActualEntry::Missing => changeset.path_operations.push(create()),

        // The path exists, but isn't a file.
        // First delete the conflicting path, then create the new file.
        //
        // A non-empty directory on the system makes the conflict operation fail at execution time,
        // as we don't want to silently wipe directory trees full of data.
        //
        // No interference with the cleanup phase: this changeset is computed after cleanup has
        // already been executed, so a directory that's cleaned up in the same run is simply gone
        // by the time we look at the system here.
        ActualEntry::Directory { .. } => {
            changeset.path_operations.push(PathOperation::Directory(
                DirectoryOperation::Conflict {
                    path: path.to_path_buf(),
                    found: FileType::Directory,
                },
            ));
            changeset.path_operations.push(create());
        }
        ActualEntry::Symlink | ActualEntry::Special => {
            // `Missing` is the only actual entry without a filetype and it's handled above.
            if let Some(found) = actual.file_type() {
                changeset
                    .path_operations
                    .push(PathOperation::File(FileOperation::Conflict {
                        path: path.to_path_buf(),
                        found,
                    }));
            }
            changeset.path_operations.push(create());
        }

        // The file exists. Check for any differences.
        ActualEntry::File { mode, owner, group } => {
            let mut modified_content = None;
            let mut modified_mode = None;
            let mut modified_owner = None;
            let mut modified_group = None;

            let actual_content = read_actual_content(path)?;
            if file.content.bytes() != actual_content.as_slice() {
                modified_content = Some(file.content.clone());
            }

            if *mode != file.mode {
                modified_mode = Some(Modified {
                    old: *mode,
                    new: file.mode,
                });
            }
            if *owner != file.owner {
                modified_owner = Some(Modified {
                    old: owner.clone(),
                    new: file.owner.clone(),
                });
            }
            if *group != file.group {
                modified_group = Some(Modified {
                    old: group.clone(),
                    new: file.group.clone(),
                });
            }

            if modified_content.is_some()
                || modified_mode.is_some()
                || modified_owner.is_some()
                || modified_group.is_some()
            {
                changeset
                    .path_operations
                    .push(PathOperation::File(FileOperation::Modify {
                        path: path.to_path_buf(),
                        content: modified_content,
                        mode: modified_mode,
                        owner: modified_owner,
                        group: modified_group,
                    }));
            }
        }
    }

    Ok(())
}

/// The type of action to perform at a given path.
///
/// This is used by [handle_paths] to decide how the directory's descendants are compared.
enum DirectoryOutcome {
    /// A usable directory (or an accepted dir-pointing symlink) is already in place
    /// and is kept, apart from possible permission fixes.
    Kept,
    /// Nothing exists at this path yet and a new directory will be created.
    Created,
    /// A conflicting actual entry will be deleted and replaced with a new directory.
    Replaced,
}

/// Compare a single directory node against its actual entry.
fn handle_directory(
    path: &std::path::Path,
    dir: &DirectoryState,
    actual: &ActualEntry,
    changeset: &mut Changeset,
) -> Result<DirectoryOutcome> {
    // Get the permissions for this directory.
    //
    // Note: It's possible for declared directories to not have any permissions set either.
    let (declared_mode, declared_owner, declared_group) =
        match dir.meta().map(|meta| &meta.permissions) {
            Some(DirectoryPermissions::Declared { mode, owner, group }) => {
                (*mode, owner.clone(), group.clone())
            }
            Some(DirectoryPermissions::Default) | None => (None, None, None),
        };

    // Set defaults for all permissions that aren't explicitly set.
    let desired_mode = declared_mode.unwrap_or(0o755);
    let desired_owner = declared_owner
        .clone()
        .unwrap_or_else(|| CURRENT_USER.clone());
    let desired_group = declared_group
        .clone()
        .unwrap_or_else(|| CURRENT_GROUP.clone());

    // Whether the user declared any permissions for this directory.
    let is_explicit =
        declared_mode.is_some() || declared_owner.is_some() || declared_group.is_some();

    let create = || {
        PathOperation::Directory(DirectoryOperation::Create {
            path: path.to_path_buf(),
            mode: desired_mode,
            owner: desired_owner.clone(),
            group: desired_group.clone(),
        })
    };

    let outcome = match actual {
        ActualEntry::Missing => {
            changeset.path_operations.push(create());
            DirectoryOutcome::Created
        }

        // The path exists, but isn't a directory.
        // Delete the conflicting path and create the directory.
        ActualEntry::File { .. } | ActualEntry::Special => {
            // `Missing` is the only actual entry without a filetype and it's handled above.
            if let Some(found) = actual.file_type() {
                changeset
                    .path_operations
                    .push(PathOperation::File(FileOperation::Conflict {
                        path: path.to_path_buf(),
                        found,
                    }));
            }
            changeset.path_operations.push(create());
            DirectoryOutcome::Replaced
        }

        // The path exists, but it's a symlink.
        ActualEntry::Symlink => {
            // Symlinks are accepted in place of non-explicit directories, as long as
            // they point towards a directory. All reads and writes of deployed
            // children simply resolve through the link.
            // Dangling symlinks and link loops also count as conflicts.
            if !is_explicit && points_to_directory(path)? {
                return Ok(DirectoryOutcome::Kept);
            }

            changeset
                .path_operations
                .push(PathOperation::File(FileOperation::Conflict {
                    path: path.to_path_buf(),
                    found: FileType::Symlink,
                }));
            changeset.path_operations.push(create());
            DirectoryOutcome::Replaced
        }

        // The directory exists. Check for any differences.
        // Only declared permission fields are enforced; everything else
        // is left alone once the directory exists.
        ActualEntry::Directory { mode, owner, group } => {
            let mut modified_mode = None;
            let mut modified_owner = None;
            let mut modified_group = None;

            if let Some(declared) = declared_mode
                && *mode != declared
            {
                modified_mode = Some(Modified {
                    old: *mode,
                    new: declared,
                });
            }
            if let Some(declared) = declared_owner
                && *owner != declared
            {
                modified_owner = Some(Modified {
                    old: owner.clone(),
                    new: declared,
                });
            }
            if let Some(declared) = declared_group
                && *group != declared
            {
                modified_group = Some(Modified {
                    old: group.clone(),
                    new: declared,
                });
            }

            if modified_mode.is_some() || modified_owner.is_some() || modified_group.is_some() {
                changeset.path_operations.push(PathOperation::Directory(
                    DirectoryOperation::Modify {
                        path: path.to_path_buf(),
                        mode: modified_mode,
                        owner: modified_owner,
                        group: modified_group,
                    },
                ));
            }
            DirectoryOutcome::Kept
        }
    };

    Ok(outcome)
}

/// Detect any packages that're missing on the current system and queue them
/// for installation.
fn handle_packages(
    desired: &State,
    system_state: &mut SystemState,
    changeset: &mut Changeset,
) -> Result<()> {
    for (manager, packages) in desired.packages.iter() {
        // We look at all installed packages, including dependencies.
        // In case some desired package has already been installed as a
        // dependency, we won't try to re-install it.
        //
        // TODO: This needs a better dependency solver.
        //   In case packages are cleaned up, we have to consider whether dependencies, which are
        //   not explicitly installed, are recursively cleaned up. Those dependencies would only
        //   be re-installed on the next run of bois.
        let installed_packages = system_state.packages(*manager)?;
        for package in packages {
            if installed_packages.contains(package) {
                continue;
            }

            changeset.package_installs.push(PackageInstall {
                manager: *manager,
                name: package.clone(),
            });
        }
    }

    Ok(())
}

/// Detect any services that aren't enabled on the current system and queue them to be enabled.
///
/// A service with the `start` flag is started at the moment it gets enabled, but if a service
/// is already enabled, it won't be started again, no matter its current status.
fn handle_services(
    desired: &State,
    system_state: &mut SystemState,
    changeset: &mut Changeset,
) -> Result<()> {
    for (manager, services) in desired.services.iter() {
        for service in services {
            if system_state.service_enabled(*manager, &service.name)? {
                continue;
            }

            changeset.service_enables.push(ServiceEnable {
                manager: *manager,
                name: service.name.clone(),
                start: service.start,
            });
        }
    }

    Ok(())
}
