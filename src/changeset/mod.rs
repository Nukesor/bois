//! This module takes care of the comparison phase.
//!
//! It creates changesets between the different states, with those states being
//! - The current config source, from which we derive the "desired state"
//! - The actual state of the system
//! - The state of a previous run (if it exists).
//!
//! As such this module contains comparison logic between those three states:
//!
//! 1. [deploy_changeset]: desired state -> actual state. The set of changes that must be executed
//!    to reach the desired state.
//! 2. [detect_drift]: previous state -> actual state. Determines drift on the system since the last
//!    deploy. Necessary to prevent accidental overrides of unadopted changes.
//! 3. [cleanup_changeset]: desired state -> previous state. Changes from the previous run that are
//!    no longer needed and must be cleaned up.
//!
//! This module only creates sets of changes, which are then later on used to report, deploy,
//! or cleanup files/packages/services/etc. by the handler logic in [`crate::handlers`].

use std::path::PathBuf;

use strum::Display;

use crate::state::{PackageManager, ServiceManager, path::FileContent};

pub mod cleanup;
pub mod deploy;
pub mod drift;

pub use cleanup::cleanup_changeset;
pub use deploy::deploy_changeset;
pub use drift::{ContentChange, Drift, PathChange, PathChangeKind, detect_drift};

/// A [`Changeset`] represents the set of all changes that're going to be
/// executed by bois to reach the desired system state.
///
/// Each kind of tasks gets executed at different points of a run.
#[derive(Debug, Default)]
pub struct Changeset {
    pub package_installs: Vec<PackageInstall>,
    pub package_uninstalls: Vec<PackageUninstall>,
    /// Create/Modify operations in the deploy phase
    /// The paths are scheduled in a root-to-leaves order.
    pub path_operations: Vec<PathOperation>,
    /// Delete operations
    /// The paths are scheduled in a leaf-to-root order.
    pub path_cleanup: Vec<PathOperation>,
    /// Services to enable during the deploy phase.
    pub service_enables: Vec<ServiceEnable>,
    /// Services to stop + disable during cleanup.
    pub service_disables: Vec<ServiceDisable>,
}

impl Changeset {
    pub fn new() -> Changeset {
        Changeset::default()
    }

    pub fn is_empty(&self) -> bool {
        self.package_installs.is_empty()
            && self.package_uninstalls.is_empty()
            && self.path_operations.is_empty()
            && self.path_cleanup.is_empty()
            && self.service_enables.is_empty()
            && self.service_disables.is_empty()
    }

    /// Merge changes of the given changeset into self.
    ///
    /// Changes are appended at the end of the respective fields, which
    /// guarantees the correct execution order.
    pub fn merge(&mut self, other: Changeset) {
        self.package_installs.extend(other.package_installs);
        self.package_uninstalls.extend(other.package_uninstalls);
        self.path_operations.extend(other.path_operations);
        self.path_cleanup.extend(other.path_cleanup);
        self.service_enables.extend(other.service_enables);
        self.service_disables.extend(other.service_disables);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInstall {
    pub manager: PackageManager,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageUninstall {
    pub manager: PackageManager,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceEnable {
    pub manager: ServiceManager,
    pub name: String,
    /// Whether the service should also be started right away when it gets
    /// enabled.
    pub start: bool,
}

/// A service that's no longer part of the desired state.
/// It gets stopped and disabled during cleanup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceDisable {
    pub manager: ServiceManager,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathOperation {
    File(FileOperation),
    Directory(DirectoryOperation),
}

impl PathOperation {
    pub fn path(&self) -> &PathBuf {
        match self {
            PathOperation::File(op) => match op {
                FileOperation::Create { path, .. }
                | FileOperation::Modify { path, .. }
                | FileOperation::Cleanup { path }
                | FileOperation::Conflict { path, .. } => path,
            },
            PathOperation::Directory(op) => match op {
                DirectoryOperation::Create { path, .. }
                | DirectoryOperation::Modify { path, .. }
                | DirectoryOperation::Cleanup { path }
                | DirectoryOperation::Conflict { path, .. } => path,
            },
        }
    }
}

/// The filetype of an actual entry on the filesystem.
///
/// Mirrors [`std::fs::FileType`], plus `Special` for sockets, devices and other special files.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum FileType {
    File,
    Directory,
    Symlink,
    #[strum(serialize = "special file")]
    Special,
}

impl FileType {
    /// Returns an emoji that represents the filetype.
    pub fn emoji(&self) -> &'static str {
        match self {
            FileType::File => "📄",
            FileType::Directory => "📁",
            FileType::Symlink => "🔗",
            FileType::Special => "✨",
        }
    }
}

/// This enum represents all possible operations for single files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileOperation {
    Create {
        path: PathBuf,
        content: FileContent,
        mode: u32,
        owner: String,
        group: String,
    },
    /// All fields on modify are optional, as not all properties necessarily need
    /// to be modified.
    Modify {
        path: PathBuf,
        content: Option<FileContent>,
        mode: Option<u32>,
        owner: Option<String>,
        group: Option<String>,
    },
    /// Delete a previously deployed file that is no longer part of the desired
    /// state.
    Cleanup { path: PathBuf },
    /// A conflict with an unmanaged non-directory actual entry has been detected.
    ///
    /// This will be reported to the user and afterwards removed.
    Conflict {
        path: PathBuf,
        /// The filetype of the actual entry that conflicts with our path.
        found: FileType,
    },
}

/// This enum represents all possible operations for directories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectoryOperation {
    Create {
        path: PathBuf,
        mode: u32,
        owner: String,
        group: String,
    },
    /// All fields on modify are optional, as not all properties necessarily need
    /// to be modified.
    Modify {
        path: PathBuf,
        mode: Option<u32>,
        owner: Option<String>,
        group: Option<String>,
    },
    /// Delete a previously deployed directory that's no longer part of the
    /// desired state.
    ///
    /// Only executed if the target is empty. As long as unmanaged files are still inside,
    /// the handler should keep the directory, but display a warning.
    Cleanup { path: PathBuf },
    /// A conflict with an unmanaged directory has been detected.
    ///
    /// This will be reported to the user and afterwards removed.
    Conflict {
        path: PathBuf,
        /// The filetype of the actual entry that conflicts with our path.
        found: FileType,
    },
}
