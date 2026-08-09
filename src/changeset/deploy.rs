//! Comparison 1: desired state -> live system.
//!
//! [deploy_changeset] determines what has to happen so the live system reaches the desired state.
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
        PackageInstall,
        PathOperation,
        system::{LiveEntry, read_live_content, read_live_entry},
    },
    constants::{CURRENT_GROUP, CURRENT_USER},
    state::{
        State,
        path::{DirectoryPermissions, DirectoryState, FileState, Node},
    },
    system_state::SystemState,
};

/// Create the changeset that transforms the live system into the desired state.
///
/// - The target [`State::path_tree`] is checked against the live filesystem
/// - Packages are checked against the cached system state.
pub fn deploy_changeset(new: &State, system_state: &mut SystemState) -> Result<Changeset> {
    let mut changeset = Changeset::new();

    handle_paths(new, &mut changeset)?;
    handle_packages(new, system_state, &mut changeset)?;

    Ok(changeset)
}

/// Recursively walk the target path tree and compare each node with the filesystem.
fn handle_paths(new: &State, changeset: &mut Changeset) -> Result<()> {
    for (path, node) in new.path_tree.flatten() {
        let live = read_live_entry(&path)?;

        match node {
            Node::File(file) => handle_file(&path, file, &live, changeset)?,
            Node::Directory(dir) => handle_directory(&path, dir, &live, changeset),
        }
    }

    Ok(())
}

fn handle_file(
    path: &std::path::Path,
    file: &FileState,
    live: &LiveEntry,
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

    match live {
        LiveEntry::Missing => changeset.path_operations.push(create()),

        // The path exists, but isn't a file.
        // First delete the conflicting path, then create the new file.
        //
        // A non-empty live directory makes the conflict operation fail at execution time, as we
        // don't want silently wipe directory trees full of data.
        //
        // No interference with the cleanup phase: this changeset is computed after cleanup has
        // already been executed, so a directory that's cleaned up in the same run is simply gone
        // by the time we look at the live system here.
        LiveEntry::Directory { .. } => {
            changeset.path_operations.push(PathOperation::Directory(
                DirectoryOperation::Conflict {
                    path: path.to_path_buf(),
                    found: FileType::Directory,
                },
            ));
            changeset.path_operations.push(create());
        }
        LiveEntry::Symlink | LiveEntry::Special => {
            // `Missing` is the only entry without a filetype and it's handled above.
            if let Some(found) = live.file_type() {
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
        LiveEntry::File { mode, owner, group } => {
            let mut modified_content = None;
            let mut modified_mode = None;
            let mut modified_owner = None;
            let mut modified_group = None;

            let live_content = read_live_content(path)?;
            if file.content.bytes() != live_content.as_slice() {
                modified_content = Some(file.content.clone());
            }

            if *mode != file.mode {
                modified_mode = Some(file.mode);
            }
            if *owner != file.owner {
                modified_owner = Some(file.owner.clone());
            }
            if *group != file.group {
                modified_group = Some(file.group.clone());
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

fn handle_directory(
    path: &std::path::Path,
    dir: &DirectoryState,
    live: &LiveEntry,
    changeset: &mut Changeset,
) {
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

    let create = || {
        PathOperation::Directory(DirectoryOperation::Create {
            path: path.to_path_buf(),
            mode: desired_mode,
            owner: desired_owner.clone(),
            group: desired_group.clone(),
        })
    };

    match live {
        LiveEntry::Missing => changeset.path_operations.push(create()),

        // The path exists, but isn't a directory.
        // Delete the conflicting path and create the directory.
        LiveEntry::File { .. } | LiveEntry::Special => {
            // `Missing` is the only entry without a filetype and it's handled above.
            if let Some(found) = live.file_type() {
                changeset
                    .path_operations
                    .push(PathOperation::File(FileOperation::Conflict {
                        path: path.to_path_buf(),
                        found,
                    }));
            }
            changeset.path_operations.push(create());
        }

        // The path exists, but it's a symlink.
        LiveEntry::Symlink => {
            changeset
                .path_operations
                .push(PathOperation::File(FileOperation::Conflict {
                    path: path.to_path_buf(),
                    found: FileType::Symlink,
                }));
            changeset.path_operations.push(create());
        }

        // The directory exists. Check for any differences.
        // Only declared permission fields are enforced; everything else
        // is left alone once the directory exists.
        LiveEntry::Directory { mode, owner, group } => {
            let mut modified_mode = None;
            let mut modified_owner = None;
            let mut modified_group = None;

            if let Some(declared) = declared_mode {
                if *mode != declared {
                    modified_mode = Some(declared);
                }
            }
            if let Some(declared) = declared_owner {
                if *owner != declared {
                    modified_owner = Some(declared);
                }
            }
            if let Some(declared) = declared_group {
                if *group != declared {
                    modified_group = Some(declared);
                }
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
        }
    }
}

/// Detect any packages that're missing on the current system and queue them
/// for installation.
fn handle_packages(
    new: &State,
    system_state: &mut SystemState,
    changeset: &mut Changeset,
) -> Result<()> {
    for (manager, packages) in new.packages.iter() {
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
            if !installed_packages.contains(package) {
                changeset.package_installs.push(PackageInstall {
                    manager: *manager,
                    name: package.clone(),
                });
            }
        }
    }

    Ok(())
}
