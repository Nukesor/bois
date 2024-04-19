use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::handlers::packages::{get_installed_packages, pacman, PackageManager};

/// This state holds all important information about the system we're running on.
///
/// It's supposed to be passed around and updated while performing operations.
/// The idea is to minimize calls to external tools such as package managers or systemd.
#[derive(Debug, Default)]
pub struct SystemState {
    pub installed_packages: HashMap<PackageManager, HashSet<String>>,
}

impl SystemState {
    pub fn new() -> Result<Self> {
        let state = Self::default();

        Ok(state)
    }

    pub fn update_packages(&mut self, manager: PackageManager) -> Result<()> {
        let list = get_installed_packages(manager)?;
        self.installed_packages.insert(manager, list);

        Ok(())
    }
}
