use crate::handlers::packages::PackageManager;

pub mod compiled_state;
pub mod state_to_host;
pub mod state_to_state;
//pub mod tree;

/// This data struct represents the set of all changes that're going to be
/// executed by bois to reach the desired system state.
///
/// This includes all possible operations for all stages.
pub type ChangeSet = Vec<Change>;

#[derive(Debug)]
pub enum Change {
    //    PathChange(PathOperation),
    PackageChange(PackageOperation),
    //    ServiceChange(ServiceOperation),
}

#[derive(Debug)]
pub enum PackageOperation {
    Remove {
        manager: PackageManager,
        name: String,
    },
    Add {
        manager: PackageManager,
        name: String,
    },
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
//
//#[derive(Debug)]
//pub enum PathOperation {
//    File(FileOperation),
//    Directory(DirectoryOperation),
//}
//
///// This enum represents all possible operations for single files.
//#[derive(Debug)]
//pub enum FileOperation {
//    Create {
//        path: PathBuf,
//        content: Vec<u8>,
//        permissions: i32,
//        owner: String,
//        group: String,
//    },
//    /// All fields on modify are optional, as not all properties necessarily need
//    /// to be modified.
//    Modify {
//        path: PathBuf,
//        content: Option<Vec<u8>>,
//        permissions: Option<i32>,
//        owner: Option<String>,
//        group: Option<String>,
//    },
//    Delete,
//}
//
///// This enum represents all possible operations for directories.
//#[derive(Debug)]
//pub enum DirectoryOperation {
//    Create {
//        path: PathBuf,
//        permissions: i32,
//        owner: String,
//        group: String,
//    },
//    /// All fields on modify are optional, as not all properties necessarily need
//    /// to be modified.
//    Modify {
//        path: PathBuf,
//        permissions: Option<i32>,
//        owner: Option<String>,
//        group: Option<String>,
//    },
//    Delete {
//        path: PathBuf,
//    },
//}
