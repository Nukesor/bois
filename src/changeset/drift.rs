//! Comparison 2: previous state -> actual state.
//!
//! Detects any drift that was introduced on the system since the last run.
//! The user might have forgotten to integrate those changes into the bois
//! config, so we inform them before the changes are overwritten or removed by
//! the deploy phase.
//!
//! Changes that are already reflected in the desired state, however, must
//! **not** be detected as drift. The user has already adopted those into the
//! config and the deploy phase will leave them untouched anyway.
//!
//! This module deliberately **doesn't** produce a [super::Changeset].
//! Nothing that's detected in here should ever be executed. It's only ever
//! displayed to the user.
//!
//! TODO: The idea is to (maybe) later on create a `bois adopt` command, which
//! would attempt to copy over changes from the system into the bois directory.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{
    changeset::FileType,
    state::{
        PackageManager,
        ServiceManager,
        State,
        path::{DirectoryPermissions, DirectoryState, FileContent, FileState, Node, Tree},
    },
    system_state::{
        SystemState,
        entry::{ActualEntry, points_to_directory, read_actual_content},
    },
};

/// Everything that changed on the system since the last run.
#[derive(Debug, Default)]
pub struct Drift {
    /// Deployed files/directories whose content, metadata or filetype changed
    /// on the system.
    pub changed_paths: Vec<PathChange>,
    /// Previously deployed files that are now missing on the system.
    pub deleted_paths: Vec<PathBuf>,
    /// Managed packages that were manually uninstalled from the system.
    pub removed_packages: BTreeMap<PackageManager, Vec<String>>,
    /// Managed services that were manually disabled on the system.
    pub disabled_services: BTreeMap<ServiceManager, Vec<String>>,
}

impl Drift {
    pub fn is_empty(&self) -> bool {
        self.changed_paths.is_empty()
            && self.deleted_paths.is_empty()
            && self.removed_packages.is_empty()
            && self.disabled_services.is_empty()
    }
}

/// A single changed path. The deployed value always comes first.
#[derive(Debug)]
pub struct PathChange {
    pub path: PathBuf,
    pub change: PathChangeKind,
}

#[derive(Debug)]
pub enum PathChangeKind {
    /// The filetype changed, e.g. a deployed file was replaced by a
    /// directory or symlink.
    FileTypeChanged {
        deployed: FileType,
        actual: FileType,
    },
    /// Content and/or metadata changed. Only the changed fields are set.
    Modified {
        /// The (unchanged) filetype of the path, for display purposes.
        filetype: FileType,
        content: Option<ContentChange>,
        /// (deployed, actual)
        mode: Option<(u32, u32)>,
        /// (deployed, actual)
        owner: Option<(String, String)>,
        /// (deployed, actual)
        group: Option<(String, String)>,
    },
}

/// The content of a file as we deployed it vs. what's on disk now.
#[derive(Debug)]
pub struct ContentChange {
    pub deployed: FileContent,
    pub actual: Vec<u8>,
}

/// Compare the previous state against the system.
///
/// The `desired` state is used to filter out changes that the user has
/// already integrated into the config.
pub fn detect_drift(
    previous: &State,
    desired: &State,
    system_state: &mut SystemState,
) -> Result<Drift> {
    let mut changes = Drift::default();

    handle_paths(&previous.path_tree, &desired.path_tree, &mut changes)?;
    handle_packages(previous, desired, system_state, &mut changes)?;
    handle_services(previous, desired, system_state, &mut changes)?;

    Ok(changes)
}

fn handle_paths(previous: &Tree, desired: &Tree, changes: &mut Drift) -> Result<()> {
    for (path, node) in previous.flatten() {
        // The respective node the desired state wants at this path, if any.
        // If this is `None`, the path is no longer desired.
        let desired_node = desired.get(&path);

        match node {
            Node::File(file) => handle_file(&path, file, desired_node, changes)?,
            Node::Directory(dir) => handle_directory(&path, dir, desired_node, changes)?,
        }
    }

    Ok(())
}

fn handle_file(
    path: &Path,
    file: &FileState,
    desired: Option<&Node>,
    changes: &mut Drift,
) -> Result<()> {
    // The file is no longer desired.
    // Since we're going to abandon/remove it anyway, any changes on the system are moot.
    let Some(desired) = desired else {
        return Ok(());
    };

    let actual = ActualEntry::read(path)?;

    // The file the desired state wants at this path, if it still does.
    let desired_file = match desired {
        Node::File(file) => Some(file),
        _ => None,
    };

    match actual {
        ActualEntry::Missing => {
            // Only report the deletion if a file is still desired at this path.
            if desired_file.is_some() {
                changes.deleted_paths.push(path.to_path_buf());
            }
        }

        ActualEntry::Directory { .. } | ActualEntry::Symlink | ActualEntry::Special => {
            // The actual entry changed type and is not a file.
            // This may be acceptable if the path has been swapped for a directory in
            // the config and the change on the system is reflected in the config.
            if actual_entry_satisfies_desired(path, desired, &actual)? {
                return Ok(());
            }

            // `Missing` is the only actual entry without a filetype and it's handled above.
            if let Some(actual) = actual.file_type() {
                changes.changed_paths.push(PathChange {
                    path: path.to_path_buf(),
                    change: PathChangeKind::FileTypeChanged {
                        deployed: FileType::File,
                        actual,
                    },
                })
            }
        }

        ActualEntry::File { mode, owner, group } => {
            // Handle the case that the desired filetype changed.
            // Due to this, the file will be replaced anyway and any changes
            // on the system are moot.
            let Some(desired_file) = desired_file else {
                return Ok(());
            };

            let actual_content = read_actual_content(path)?;

            // Check for differences in the actual file content.
            // If any are detected, they're only reported if the actual state differs from
            // the desired state.
            let content_adopted = desired_file.content.bytes() == actual_content.as_slice();
            let mode_adopted = desired_file.mode == mode;
            let owner_adopted = desired_file.owner == owner;
            let group_adopted = desired_file.group == group;

            let content = (file.content.bytes() != actual_content.as_slice() && !content_adopted)
                .then(|| ContentChange {
                    deployed: file.content.clone(),
                    actual: actual_content,
                });
            let mode = (mode != file.mode && !mode_adopted).then_some((file.mode, mode));
            let owner = (owner != file.owner && !owner_adopted)
                .then(|| (file.owner.clone(), owner.clone()));
            let group = (group != file.group && !group_adopted)
                .then(|| (file.group.clone(), group.clone()));

            // Only report the path if at least one field changed to a value that isn't desired.
            if content.is_some() || mode.is_some() || owner.is_some() || group.is_some() {
                changes.changed_paths.push(PathChange {
                    path: path.to_path_buf(),
                    change: PathChangeKind::Modified {
                        filetype: FileType::File,
                        content,
                        mode,
                        owner,
                        group,
                    },
                });
            }
        }
    }

    Ok(())
}

fn handle_directory(
    path: &Path,
    dir: &DirectoryState,
    desired: Option<&Node>,
    changes: &mut Drift,
) -> Result<()> {
    // The directory is no longer desired.
    // Since we're going to abandon/remove it anyway, any changes on the system are moot.
    let Some(desired) = desired else {
        return Ok(());
    };

    // Implicit directories aren't managed: their existence and metadata are
    // none of our business.
    let Some(meta) = dir.meta() else {
        return Ok(());
    };

    let (declared_mode, declared_owner, declared_group) = match &meta.permissions {
        DirectoryPermissions::Declared { mode, owner, group } => (*mode, owner, group),
        DirectoryPermissions::Default => (None, &None, &None),
    };

    // The directory the desired state wants at this path, if it still wants one.
    let desired_dir = match desired {
        Node::Directory(dir) => Some(dir),
        _ => None,
    };
    // The desired state's declared permissions for this path.
    let (desired_mode, desired_owner, desired_group) = match desired_dir
        .and_then(|dir| dir.meta())
        .map(|meta| &meta.permissions)
    {
        Some(DirectoryPermissions::Declared { mode, owner, group }) => {
            (*mode, owner.as_ref(), group.as_ref())
        }
        _ => (None, None, None),
    };

    let actual = ActualEntry::read(path)?;

    match actual {
        ActualEntry::Missing => {
            // Only report the deletion if a directory is still desired at this path.
            if desired_dir.is_some() {
                changes.deleted_paths.push(path.to_path_buf());
            }
        }

        ActualEntry::File { .. } | ActualEntry::Symlink | ActualEntry::Special => {
            // The actual entry changed type and is no longer a directory.
            // This may be acceptable if the path has been swapped for a file in
            // the config and the change on the system is reflected in the config.
            if actual_entry_satisfies_desired(path, desired, &actual)? {
                return Ok(());
            }

            // `Missing` is the only actual entry without a filetype and it's handled above.
            if let Some(actual) = actual.file_type() {
                changes.changed_paths.push(PathChange {
                    path: path.to_path_buf(),
                    change: PathChangeKind::FileTypeChanged {
                        deployed: FileType::Directory,
                        actual,
                    },
                })
            }
        }

        ActualEntry::Directory {
            mode: actual_mode,
            owner: actual_owner,
            group: actual_group,
        } => {
            // Handle the case that the desired filetype changed.
            // Due to this, the directory will be replaced anyway and any
            // changes on the system are moot.
            if desired_dir.is_none() {
                return Ok(());
            }

            // Only explicitly declared permissions are managed by us.
            // Check which of these no longer match the actual state and
            // also don't match the desired state.
            let mode = declared_mode
                .filter(|declared| actual_mode != *declared)
                .filter(|_| desired_mode.is_some_and(|d| d != actual_mode))
                .map(|declared| (declared, actual_mode));
            let owner = declared_owner
                .as_ref()
                .filter(|declared| &actual_owner != *declared)
                .filter(|_| desired_owner.is_some_and(|d| d != &actual_owner))
                .map(|declared| (declared.clone(), actual_owner.clone()));
            let group = declared_group
                .as_ref()
                .filter(|declared| &actual_group != *declared)
                .filter(|_| desired_group.is_some_and(|d| d != &actual_group))
                .map(|declared| (declared.clone(), actual_group.clone()));

            if mode.is_some() || owner.is_some() || group.is_some() {
                changes.changed_paths.push(PathChange {
                    path: path.to_path_buf(),
                    change: PathChangeKind::Modified {
                        filetype: FileType::Directory,
                        content: None,
                        mode,
                        owner,
                        group,
                    },
                });
            }
        }
    }

    Ok(())
}

/// Whether the deploy phase will keep the actual entry at this path in place.
///
/// Only called for nodes whose actual filetype no longer matches the previously deployed one.
/// This is used to filter out filetype changes that were already (at least partially) adopted
/// into the config, e.g. a deployed file that was manually replaced with a directory while the
/// desired state also wants a directory.
///
/// Content and metadata of nodes isn't inspected here. That's done during the deploy step.
fn actual_entry_satisfies_desired(
    path: &Path,
    desired: &Node,
    actual: &ActualEntry,
) -> Result<bool> {
    let satisfied = match (desired, actual) {
        // A desired file/directory is satisfied by any actual entry of the same filetype.
        (Node::File(_), ActualEntry::File { .. })
        | (Node::Directory(_), ActualEntry::Directory { .. }) => true,
        // A desired directory is satisfied by a dir-pointing symlink, unless permissions are
        // declared. This mirrors the deploy comparison's symlink handling.
        (Node::Directory(dir), ActualEntry::Symlink) => {
            let is_explicit = match dir.meta().map(|meta| &meta.permissions) {
                Some(DirectoryPermissions::Declared { mode, owner, group }) => {
                    mode.is_some() || owner.is_some() || group.is_some()
                }
                _ => false,
            };
            !is_explicit && points_to_directory(path)?
        }
        _ => false,
    };

    Ok(satisfied)
}

/// Report any packages that were deployed, are still desired, but have been
/// manually removed from the system since the last deploy.
fn handle_packages(
    previous: &State,
    desired: &State,
    system_state: &mut SystemState,
    changes: &mut Drift,
) -> Result<()> {
    for (manager, previous_packages) in previous.packages.iter() {
        // We compare against all installed packages including dependencies.
        // A package that was demoted to a dependency is still on the system
        // and thereby not drift.
        let installed = system_state.packages(*manager)?;

        for package in previous_packages {
            if installed.contains(package) {
                continue;
            }

            // Only report the removal if the package is still desired.
            // Otherwise its removal is simply an already-done cleanup.
            let still_desired = desired
                .packages
                .get(manager)
                .is_some_and(|packages| packages.contains(package));

            if still_desired {
                changes
                    .removed_packages
                    .entry(*manager)
                    .or_default()
                    .push(package.clone());
            }
        }
    }

    Ok(())
}

/// Report any services that were deployed, are still desired, but have been
/// manually disabled on the system since the last deploy.
fn handle_services(
    previous: &State,
    desired: &State,
    system_state: &mut SystemState,
    changes: &mut Drift,
) -> Result<()> {
    for (manager, previous_services) in previous.services.iter() {
        for service in previous_services {
            if system_state.service_enabled(*manager, &service.name)? {
                continue;
            }

            // Only report the disable if the service is still desired.
            // Otherwise it's simply an already-done cleanup.
            let still_desired = desired.services.get(manager).is_some_and(|services| {
                services.iter().any(|desired| desired.name == service.name)
            });
            if still_desired {
                changes
                    .disabled_services
                    .entry(*manager)
                    .or_default()
                    .push(service.name.clone());
            }
        }
    }

    Ok(())
}
