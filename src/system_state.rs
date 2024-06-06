use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::handlers::packages::{get_detected_groups, pacman, paru, PackageManager};

/// This state holds all important information about the system we're running on.
///
/// It's supposed to be passed around and updated while performing operations.
/// The idea is to minimize calls to external tools such as package managers or systemd.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SystemState {
    packages: HashMap<PackageManager, HashSet<String>>,
    explicit_packages: HashMap<PackageManager, HashSet<String>>,
    detected_package_groups: HashMap<PackageManager, HashSet<String>>,
}

impl SystemState {
    pub fn new() -> Result<Self> {
        let state = Self::default();

        Ok(state)
    }

    /// Get all installed packages for the current system, which includes potential dependencies.
    ///
    /// This list is cached, however if the cache isn't loaded yet it'll be retrieved.
    pub fn packages(&mut self, manager: PackageManager) -> Result<HashSet<String>> {
        let list = if let Some(packages) = self.packages.get(&manager) {
            packages.clone()
        } else {
            self.update_packages(manager)?;
            self.packages.get(&manager).unwrap().clone()
        };

        Ok(list)
    }

    /// Get all **explicitly** installed packages for the current system.
    ///
    /// This is list is cached, however if the cache isn't set yet, load the list.
    pub fn explicit_packages(&mut self, manager: PackageManager) -> Result<HashSet<String>> {
        let list = if let Some(packages) = self.explicit_packages.get(&manager) {
            packages.clone()
        } else {
            self.update_packages(manager)?;
            self.explicit_packages.get(&manager).unwrap().clone()
        };

        Ok(list)
    }

    /// Update the installed packages, both explicit and not explicit.
    pub fn update_packages(&mut self, manager: PackageManager) -> Result<()> {
        // Get a list of all installed packages on the system.
        let all_packages = match manager {
            PackageManager::Pacman => pacman::packages()?,
            // Paru doesn't allow dependencies from the AUR, so we only have to care
            // about explicit packages.
            PackageManager::Paru => paru::explicit_packages()?,
            PackageManager::Apt => todo!(),
        };

        // Get a list of all packages that were **explicitly** installed on the system.
        let explicit_packages = match manager {
            PackageManager::Pacman => pacman::explicit_packages()?,
            PackageManager::Paru => all_packages.clone(),
            PackageManager::Apt => todo!(),
        };

        self.packages.insert(manager, all_packages);
        self.explicit_packages.insert(manager, explicit_packages);

        Ok(())
    }

    /// Get all installed packages for the current system.
    ///
    /// This is list is cached, however if the cache isn't set yet, load the list.
    pub fn detected_package_groups(&mut self, manager: PackageManager) -> Result<HashSet<String>> {
        let list = if let Some(packages) = self.detected_package_groups.get(&manager) {
            packages.clone()
        } else {
            let list = get_detected_groups(manager)?;
            self.detected_package_groups.insert(manager, list);
            self.detected_package_groups.get(&manager).unwrap().clone()
        };

        Ok(list)
    }
}
