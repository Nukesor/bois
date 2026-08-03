use std::collections::BTreeSet;

use anyhow::Result;

use crate::{state::PackageManager, system_state::package_managers::SystemPackages};

pub mod package_managers;

/// This state holds all important information about the system we're running on.
///
/// It's supposed to be passed around and updated while performing operations.
/// The idea is to minimize calls to external tools such as package managers or
/// systemd: every query result is cached.
///
/// File information is deliberately **not** part of this state. The filesystem
/// is queried live per path during comparisons, as we cannot pull the whole
/// filesystem into a state struct.
#[derive(Debug)]
pub struct SystemState {
    packages: SystemPackages,
}

impl SystemState {
    pub fn new() -> Result<Self> {
        Ok(SystemState {
            packages: SystemPackages::default(),
        })
    }

    /// Get all installed packages for the current system, which includes potential dependencies.
    pub fn packages(&mut self, manager: PackageManager) -> Result<&BTreeSet<String>> {
        self.packages.packages(manager)
    }

    /// Get all **explicitly** installed packages for the current system.
    pub fn explicit_packages(&mut self, manager: PackageManager) -> Result<&BTreeSet<String>> {
        self.packages.explicit_packages(manager)
    }

    /// Re-query the installed packages, e.g. after packages were un-/installed.
    pub fn update_packages(&mut self, manager: PackageManager) -> Result<()> {
        self.packages.update_packages(manager)
    }

    /// Get the names of all package *groups* (e.g. pacman's `base-devel`) that
    /// exist on the system.
    pub fn package_groups(&mut self, manager: PackageManager) -> Result<&BTreeSet<String>> {
        self.packages.package_groups(manager)
    }

    /// Get the list of packages a package *group* consists of.
    pub fn packages_for_group(
        &mut self,
        manager: PackageManager,
        group: &str,
    ) -> Result<BTreeSet<String>> {
        self.packages.packages_for_group(manager, group)
    }
}
