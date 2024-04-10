use std::collections::HashMap;

use crate::handlers::packages::PackageManager;

/// This state holds all important information about the system we're running on.
///
/// It's supposed to be passed around and updated while performing operations.
/// The idea is to minimize calls to external tools such as package managers or systemd.
#[derive(Debug, Default)]
pub struct SystemState {
    pub installed_packages: HashMap<PackageManager, Vec<String>>,
}
