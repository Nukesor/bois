use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::handlers::packages::{get_detected_groups, get_installed_packages, PackageManager};

/// This state holds all important information about the system we're running on.
///
/// It's supposed to be passed around and updated while performing operations.
/// The idea is to minimize calls to external tools such as package managers or systemd.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SystemState {
    installed_packages: HashMap<PackageManager, HashSet<String>>,
    detected_package_groups: HashMap<PackageManager, HashSet<String>>,
}

impl SystemState {
    pub fn new() -> Result<Self> {
        let state = Self::default();

        Ok(state)
    }

    /// Get all installed packages for the current system.
    ///
    /// This is list is cached, however if the cache isn't set yet, load the list.
    pub fn installed_packages(&mut self, manager: PackageManager) -> Result<HashSet<String>> {
        let list = if let Some(packages) = self.installed_packages.get(&manager) {
            packages.clone()
        } else {
            self.update_packages(manager)?;
            self.installed_packages.get(&manager).unwrap().clone()
        };

        Ok(list)
    }

    pub fn update_packages(&mut self, manager: PackageManager) -> Result<()> {
        let list = get_installed_packages(manager, false)?;
        self.installed_packages.insert(manager, list);

        Ok(())
    }

    /// Get all installed packages for the current system.
    ///
    /// This is list is cached, however if the cache isn't set yet, load the list.
    pub fn detected_package_groups(&mut self, manager: PackageManager) -> Result<HashSet<String>> {
        let list = if let Some(packages) = self.detected_package_groups.get(&manager) {
            packages.clone()
        } else {
            self.update_detect_groups(manager)?;
            self.detected_package_groups.get(&manager).unwrap().clone()
        };

        Ok(list)
    }

    pub fn update_detect_groups(&mut self, manager: PackageManager) -> Result<()> {
        let list = get_detected_groups(manager)?;
        self.detected_package_groups.insert(manager, list);

        Ok(())
    }
}
