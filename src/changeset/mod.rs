use std::path::PathBuf;

use crate::handlers::packages::PackageManager;

pub mod compiled_state;
pub mod helper;
pub mod host_to_state;
pub mod state_to_host;
pub mod state_to_state;
pub mod tree;

/// This data struct represents the set of all changes that're going to be
/// executed by bois to reach the desired system state.
///
/// This includes all possible operations for all stages.
#[derive(Debug, Default)]
pub struct Changeset {
    pub package_installs: Vec<PackageInstall>,
    pub package_uninstalls: Vec<PackageUninstall>,
    pub path_operations: Vec<PathOperation>,
}

impl Changeset {
    pub fn new() -> Changeset {
        Changeset::default()
    }

    pub fn is_empty(&self) -> bool {
        self.package_installs.is_empty()
            && self.package_uninstalls.is_empty()
            && self.path_operations.is_empty()
    }

    /// Merge changes of the given changeset into self.
    /// Changes are appended at the end, which guarantees the correct
    /// execution order.
    pub fn merge(&mut self, other: Changeset) {
        self.package_installs.extend(other.package_installs);
        self.package_uninstalls.extend(other.package_uninstalls);
        self.path_operations.extend(other.path_operations);
    }
}

#[derive(Debug)]
pub struct PackageUninstall {
    pub manager: PackageManager,
    pub name: String,
}

#[derive(Debug)]
pub struct PackageInstall {
    pub manager: PackageManager,
    pub name: String,
}

//#[derive(Debug)]
//pub enum ServiceOperation {
//    Enable {
//        manager: ServiceManager,
//        name: String,
//    },
//    Disable {
//        manager: ServiceManager,
//        name: String,
//    },
//}

#[derive(Debug)]
pub enum PathOperation {
    File(FileOperation),
    Directory(DirectoryOperation),
}

/// This enum represents all possible operations for single files.
#[derive(Debug)]
pub enum FileOperation {
    Create {
        path: PathBuf,
        content: Vec<u8>,
        mode: u32,
        owner: String,
        group: String,
    },
    /// All fields on modify are optional, as not all properties necessarily need
    /// to be modified.
    Modify {
        path: PathBuf,
        content: Option<Vec<u8>>,
        mode: Option<u32>,
        owner: Option<String>,
        group: Option<String>,
    },
    Delete {
        path: PathBuf,
    },
}

/// This enum represents all possible operations for directories.
#[derive(Debug)]
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
    Delete {
        path: PathBuf,
    },
}
