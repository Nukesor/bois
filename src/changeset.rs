use std::path::PathBuf;

/// This data struct represents the set of all changes that're going to be
/// executed by bois to reach the desired system state.
///
/// This includes all possible operations for all stages.
struct ChangeSet {
    //pub service_changes: Vec<ServiceChange>,
    //pub package_changes: Vec<PackageChange>,
    pub file_changes: Vec<PathChange>,
}

pub struct PathChange {
    path: PathBuf,
    operation: Operation,
}

enum Operation {
    FileOperation(FileOperation),
    DirectoryOperation(DirectoryOperation),
}

/// This enum represents all possible operations for single files.
pub enum FileOperation {
    Create {
        content: Vec<u8>,
        permissions: i32,
        owner: String,
        group: String,
    },
    /// All fields on modify are optional, as not all properties necessarily need
    /// to be modified.
    Modify {
        content: Option<Vec<u8>>,
        permissions: Option<i32>,
        owner: Option<String>,
        group: Option<String>,
    },
    Delete,
}

/// This enum represents all possible operations for directories.
pub enum DirectoryOperation {
    Create {
        permissions: i32,
        owner: String,
        group: String,
    },
    /// All fields on modify are optional, as not all properties necessarily need
    /// to be modified.
    Modify {
        permissions: Option<i32>,
        owner: Option<String>,
        group: Option<String>,
    },
    Delete,
}
