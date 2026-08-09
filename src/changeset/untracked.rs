//! Comparison 2: last-deployed state -> live system.
//!
//! Detects any untracked changes that were made to the system since the last
//! deployment. The user might have forgotten to integrate those changes into
//! the bois config, so we inform them before the changes are overwritten or
//! removed by the deployment.
//!
//! This deliberately **doesn't** produce a [super::Changeset].
//! Nothing in here should ever be executed. It's only displayed to the user.
//!
//! TODO: The idea is to (maybe) later on create an `bois absorb` command, which
//! would go attempt to copy over on-system changes to bude directory.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{
    changeset::{
        FileType,
        system::{LiveEntry, read_live_content, read_live_entry},
    },
    state::{
        PackageManager,
        State,
        path::{DirectoryPermissions, DirectoryState, FileContent, FileState, Node},
    },
    system_state::SystemState,
};

/// Everything that changed on the live system since the last deployment.
#[derive(Debug, Default)]
pub struct UntrackedChanges {
    /// Deployed files/directories whose content, metadata or filetype changed
    /// on the system.
    pub changed_paths: Vec<PathChange>,
    /// Previously deployed files that are now missing on the system.
    pub deleted_paths: Vec<PathBuf>,
    /// Managed packages that were manually uninstalled from the system.
    pub removed_packages: Vec<(PackageManager, String)>,
}

impl UntrackedChanges {
    pub fn is_empty(&self) -> bool {
        self.changed_paths.is_empty()
            && self.deleted_paths.is_empty()
            && self.removed_packages.is_empty()
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
/// `new` (the currently desired state) is only used to filter out scenarios where something
/// has been removed manually, but is also no longer tracked by bois.
/// In that case, we don't report the manual removal as drift.
pub fn detect_untracked_changes(
    old: &State,
    new: &State,
    system_state: &mut SystemState,
) -> Result<UntrackedChanges> {
    let mut changes = UntrackedChanges::default();

    handle_paths(old, &mut changes)?;
    handle_packages(old, new, system_state, &mut changes)?;

    Ok(changes)
}

fn handle_paths(old: &State, changes: &mut UntrackedChanges) -> Result<()> {
    for (path, node) in old.path_tree.flatten() {
        match node {
            Node::File(file) => handle_file(&path, file, changes)?,
            Node::Directory(dir) => handle_directory(&path, dir, changes)?,
        }
    }

    Ok(())
}

fn handle_file(path: &Path, file: &FileState, changes: &mut UntrackedChanges) -> Result<()> {
    let live = read_live_entry(path)?;

    match live {
        LiveEntry::Missing => changes.deleted_paths.push(path.to_path_buf()),

        LiveEntry::Directory { .. } | LiveEntry::Symlink | LiveEntry::Special => {
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
            let live_content = read_live_content(path)?;

            let content =
                (file.content.bytes() != live_content.as_slice()).then(|| ContentChange {
                    deployed: file.content.clone(),
                    live: live_content,
                });
            let mode = (mode != file.mode).then_some((file.mode, mode));
            let owner = (owner != file.owner).then(|| (file.owner.clone(), owner.clone()));
            let group = (group != file.group).then(|| (file.group.clone(), group.clone()));

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
    changes: &mut UntrackedChanges,
) -> Result<()> {
    // Implicit directories aren't managed: their existence and metadata are
    // none of our business.
    let Some(meta) = dir.meta() else {
        return Ok(());
    };

    // The per-field permission declarations, if there are any. Only declared
    // fields are actively managed, so only they can drift.
    let (declared_mode, declared_owner, declared_group) = match &meta.permissions {
        DirectoryPermissions::Declared { mode, owner, group } => (*mode, owner, group),
        DirectoryPermissions::Default => (None, &None, &None),
    };

    let live = read_live_entry(path)?;

    match live {
        LiveEntry::Missing => changes.deleted_paths.push(path.to_path_buf()),
        LiveEntry::File { .. } | LiveEntry::Special => {
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

        LiveEntry::Symlink => {
            changes.changed_paths.push(PathChange {
                path: path.to_path_buf(),
                change: PathChangeKind::FileTypeChanged {
                    deployed: FileType::Directory,
                    live: FileType::Symlink,
                },
            });
        }

        LiveEntry::Directory {
            mode: live_mode,
            owner: live_owner,
            group: live_group,
        } => {
            // Undeclared fields were derived from defaults and aren't
            // actively managed, so their drift isn't reported.
            let mode = declared_mode
                .filter(|declared| live_mode != *declared)
                .map(|declared| (declared, live_mode));
            let owner = declared_owner
                .as_ref()
                .filter(|declared| &live_owner != *declared)
                .map(|declared| (declared.clone(), live_owner.clone()));
            let group = declared_group
                .as_ref()
                .filter(|declared| &live_group != *declared)
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

/// Report any packages that were deployed, are still desired, but have been
/// manually removed from the system since the last deploy.
fn handle_packages(
    old: &State,
    new: &State,
    system_state: &mut SystemState,
    changes: &mut UntrackedChanges,
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
