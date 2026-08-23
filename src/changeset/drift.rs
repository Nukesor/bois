//! Comparison 2: last-deployed state -> live system.
//!
//! Detects any drift that was introduced on the system since the last
//! deployment. The user might have forgotten to integrate those changes into
//! the bois config, so we inform them before the changes are overwritten or
//! removed by the deployment.
//!
//! Changes that are already reflected in the new desired state, however, must
//! **not** be detected as drift. The user has already absorbed those into the
//! config and the deployment will leave them untouched anyway.
//!
//! This module deliberately **doesn't** produce a [super::Changeset].
//! Nothing that's detected in here should ever be executed. It's only ever
//! displayed to the user.
//!
//! TODO: The idea is to (maybe) later on create a `bois absorb` command, which
//! would attempt to copy over on-system changes to the bois directory.

use std::path::{Path, PathBuf};

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
        entry::{LiveEntry, points_to_directory, read_live_content},
    },
};

/// Everything that changed on the live system since the last deployment.
#[derive(Debug, Default)]
pub struct Drift {
    /// Deployed files/directories whose content, metadata or filetype changed
    /// on the system.
    pub changed_paths: Vec<PathChange>,
    /// Previously deployed files that are now missing on the system.
    pub deleted_paths: Vec<PathBuf>,
    /// Managed packages that were manually uninstalled from the system.
    pub removed_packages: Vec<(PackageManager, String)>,
    /// Managed services that were manually disabled on the system.
    pub disabled_services: Vec<(ServiceManager, String)>,
}

impl Drift {
    pub fn is_empty(&self) -> bool {
        self.changed_paths.is_empty()
            && self.deleted_paths.is_empty()
            && self.removed_packages.is_empty()
            && self.disabled_services.is_empty()
    }
}

/// A single changed path. The old (deployed) value always comes first.
#[derive(Debug)]
pub struct PathChange {
    pub path: PathBuf,
    pub change: PathChangeKind,
}

#[derive(Debug)]
pub enum PathChangeKind {
    /// The filetype changed, e.g. a deployed file was replaced by a
    /// directory or symlink.
    FileTypeChanged { deployed: FileType, live: FileType },
    /// Content and/or metadata changed. Only the changed fields are set.
    Modified {
        content: Option<ContentChange>,
        /// (deployed, live)
        mode: Option<(u32, u32)>,
        /// (deployed, live)
        owner: Option<(String, String)>,
        /// (deployed, live)
        group: Option<(String, String)>,
    },
}

/// The content of a file as we deployed it vs. what's on disk now.
#[derive(Debug)]
pub struct ContentChange {
    pub deployed: FileContent,
    pub live: Vec<u8>,
}

/// Compare the last-deployed state with the live system.
///
/// The `new` (desired) state is used to filter out changes that the user has
/// already integrated into the config.
pub fn detect_drift(old: &State, new: &State, system_state: &mut SystemState) -> Result<Drift> {
    let mut changes = Drift::default();

    handle_paths(&old.path_tree, &new.path_tree, &mut changes)?;
    handle_packages(old, new, system_state, &mut changes)?;
    handle_services(old, new, system_state, &mut changes)?;

    Ok(changes)
}

fn handle_paths(old: &Tree, new: &Tree, changes: &mut Drift) -> Result<()> {
    for (path, node) in old.flatten() {
        // The respective node the new state wants at this path, if any.
        // If this is `None`, the path is no longer desired.
        let desired = new.get(&path);

        match node {
            Node::File(file) => handle_file(&path, file, desired, changes)?,
            Node::Directory(dir) => handle_directory(&path, dir, desired, changes)?,
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
    // Since we're going to abandon/remove it anyway, any on-system changes are moot.
    let Some(desired) = desired else {
        return Ok(());
    };

    let live = LiveEntry::read(path)?;

    // The file the new state desires at this path, if it still does.
    let desired_file = match desired {
        Node::File(file) => Some(file),
        _ => None,
    };

    match live {
        LiveEntry::Missing => {
            // Only report the deletion if a file is still desired at this path.
            if desired_file.is_some() {
                changes.deleted_paths.push(path.to_path_buf());
            }
        }

        LiveEntry::Directory { .. } | LiveEntry::Symlink | LiveEntry::Special => {
            // The on-system entry changed type and is not a file.
            // This may be acceptable if the path has been swapped for a directory in
            // the config and the on-system change is reflected in the config.
            if live_entry_satisfies_desired(path, desired, &live)? {
                return Ok(());
            }

            // `Missing` is the only entry without a filetype and it's handled above.
            if let Some(live) = live.file_type() {
                changes.changed_paths.push(PathChange {
                    path: path.to_path_buf(),
                    change: PathChangeKind::FileTypeChanged {
                        deployed: FileType::File,
                        live,
                    },
                })
            }
        }

        LiveEntry::File { mode, owner, group } => {
            // Handle the case that the desired filetype changed.
            // Due to this, the file will be replaced anyway and any on-system
            // changes are moot.
            let Some(desired_file) = desired_file else {
                return Ok(());
            };

            let live_content = read_live_content(path)?;

            // Check for differences in the actual file content.
            // If any are detected, they're only reported if the new on-system state differs from
            // the desired state.
            let content_absorbed = desired_file.content.bytes() == live_content.as_slice();
            let mode_absorbed = desired_file.mode == mode;
            let owner_absorbed = desired_file.owner == owner;
            let group_absorbed = desired_file.group == group;

            let content = (file.content.bytes() != live_content.as_slice() && !content_absorbed)
                .then(|| ContentChange {
                    deployed: file.content.clone(),
                    live: live_content,
                });
            let mode = (mode != file.mode && !mode_absorbed).then_some((file.mode, mode));
            let owner = (owner != file.owner && !owner_absorbed)
                .then(|| (file.owner.clone(), owner.clone()));
            let group = (group != file.group && !group_absorbed)
                .then(|| (file.group.clone(), group.clone()));

            // Only report the path if at least one field changed to a value that isn't desired.
            if content.is_some() || mode.is_some() || owner.is_some() || group.is_some() {
                changes.changed_paths.push(PathChange {
                    path: path.to_path_buf(),
                    change: PathChangeKind::Modified {
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
    // Since we're going to abandon/remove it anyway, any on-system changes are moot.
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

    // The directory the new state wants at this path, if it still wants one.
    let desired_dir = match desired {
        Node::Directory(dir) => Some(dir),
        _ => None,
    };
    // The new state's declared permissions for this path.
    let (desired_mode, desired_owner, desired_group) = match desired_dir
        .and_then(|dir| dir.meta())
        .map(|meta| &meta.permissions)
    {
        Some(DirectoryPermissions::Declared { mode, owner, group }) => {
            (*mode, owner.as_ref(), group.as_ref())
        }
        _ => (None, None, None),
    };

    let live = LiveEntry::read(path)?;

    match live {
        LiveEntry::Missing => {
            // Only report the deletion if a directory is still desired at this path.
            if desired_dir.is_some() {
                changes.deleted_paths.push(path.to_path_buf());
            }
        }

        LiveEntry::File { .. } | LiveEntry::Symlink | LiveEntry::Special => {
            // The on-system entry changed type and is no longer a directory.
            // This may be acceptable if the path has been swapped for a file in
            // the config and the on-system change is reflected in the config.
            if live_entry_satisfies_desired(path, desired, &live)? {
                return Ok(());
            }

            // `Missing` is the only entry without a filetype and it's handled above.
            if let Some(live) = live.file_type() {
                changes.changed_paths.push(PathChange {
                    path: path.to_path_buf(),
                    change: PathChangeKind::FileTypeChanged {
                        deployed: FileType::Directory,
                        live,
                    },
                })
            }
        }

        LiveEntry::Directory {
            mode: live_mode,
            owner: live_owner,
            group: live_group,
        } => {
            // Handle the case that the desired filetype changed.
            // Due to this, the directory will be replaced anyway and any
            // on-system changes are moot.
            if desired_dir.is_none() {
                return Ok(());
            }

            // Only explicitly declared permissions are managed by us.
            // Check which of these no longer match the on-system state and
            // also don't match the desired state.
            let mode = declared_mode
                .filter(|declared| live_mode != *declared)
                .filter(|_| desired_mode.is_some_and(|d| d != live_mode))
                .map(|declared| (declared, live_mode));
            let owner = declared_owner
                .as_ref()
                .filter(|declared| &live_owner != *declared)
                .filter(|_| desired_owner.is_some_and(|d| d != &live_owner))
                .map(|declared| (declared.clone(), live_owner.clone()));
            let group = declared_group
                .as_ref()
                .filter(|declared| &live_group != *declared)
                .filter(|_| desired_group.is_some_and(|d| d != &live_group))
                .map(|declared| (declared.clone(), live_group.clone()));

            if mode.is_some() || owner.is_some() || group.is_some() {
                changes.changed_paths.push(PathChange {
                    path: path.to_path_buf(),
                    change: PathChangeKind::Modified {
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

/// Whether the deployment will keep the live entry at this path in place.
///
/// Only called for entries whose live filetype no longer matches the previously deployed one.
/// This is used to filter out filetype changes that were already (at least partially) absorbed
/// into the config, e.g. a deployed file that was manually replaced with a directory while the
/// new state also desires a directory.
///
/// Content and metadata of entries isn't inspected here. That's done during the deploy step.
fn live_entry_satisfies_desired(path: &Path, desired: &Node, live: &LiveEntry) -> Result<bool> {
    let satisfied = match (desired, live) {
        // A desired file/directory is satisfied by any live entry of the same filetype.
        (Node::File(_), LiveEntry::File { .. })
        | (Node::Directory(_), LiveEntry::Directory { .. }) => true,
        // A desired directory is satisfied by a dir-pointing symlink, unless permissions are
        // declared. This mirrors the deploy comparison's symlink handling.
        (Node::Directory(dir), LiveEntry::Symlink) => {
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
    old: &State,
    new: &State,
    system_state: &mut SystemState,
    changes: &mut Drift,
) -> Result<()> {
    for (manager, old_packages) in old.packages.iter() {
        // We compare against all installed packages including dependencies.
        // A package that was demoted to a dependency is still on the system
        // and thereby not drift.
        let installed = system_state.packages(*manager)?;

        for package in old_packages {
            if installed.contains(package) {
                continue;
            }

            // Only report the removal if the package is still desired.
            // Otherwise its removal is simply an already-done cleanup.
            let still_desired = new
                .packages
                .get(manager)
                .is_some_and(|packages| packages.contains(package));
            if still_desired {
                changes.removed_packages.push((*manager, package.clone()));
            }
        }
    }

    Ok(())
}

/// Report any services that were deployed, are still desired, but have been
/// manually disabled on the system since the last deploy.
fn handle_services(
    old: &State,
    new: &State,
    system_state: &mut SystemState,
    changes: &mut Drift,
) -> Result<()> {
    for (manager, old_services) in old.services.iter() {
        for service in old_services {
            if system_state.service_enabled(*manager, &service.name)? {
                continue;
            }

            // Only report the disable if the service is still desired.
            // Otherwise it's simply an already-done cleanup.
            let still_desired = new
                .services
                .get(manager)
                .is_some_and(|services| services.iter().any(|new| new.name == service.name));
            if still_desired {
                changes
                    .disabled_services
                    .push((*manager, service.name.clone()));
            }
        }
    }

    Ok(())
}
