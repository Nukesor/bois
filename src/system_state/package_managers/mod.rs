use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use crate::state::PackageManager;

mod pacman;
mod paru;

/// A lazy cache for package manager queries.
///
/// All lists are only retrieved once and cached afterwards, so we don't
/// repeatedly shell out to package managers during a single run.
#[derive(Debug, Default)]
pub struct SystemPackages {
    packages: BTreeMap<PackageManager, BTreeSet<String>>,
    explicit_packages: BTreeMap<PackageManager, BTreeSet<String>>,
    package_groups: BTreeMap<PackageManager, BTreeSet<String>>,
}

impl SystemPackages {
    /// Get all installed packages for the current system, which includes potential dependencies.
    pub fn packages(&mut self, manager: PackageManager) -> Result<&BTreeSet<String>> {
        if !self.packages.contains_key(&manager) {
            self.update_packages(manager)?;
        }
        Ok(self.packages.get(&manager).unwrap())
    }

    /// Get all **explicitly** installed packages for the current system.
    pub fn explicit_packages(&mut self, manager: PackageManager) -> Result<&BTreeSet<String>> {
        if !self.explicit_packages.contains_key(&manager) {
            self.update_packages(manager)?;
        }
        Ok(self.explicit_packages.get(&manager).unwrap())
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

    /// Get the set of all package *groups* (e.g. pacman's `base-devel`) that
    /// are known to the system's package database.
    pub fn package_groups(&mut self, manager: PackageManager) -> Result<&BTreeSet<String>> {
        if let std::collections::btree_map::Entry::Vacant(entry) =
            self.package_groups.entry(manager)
        {
            let list = match manager {
                PackageManager::Pacman => pacman::detect_installed_groups()?,
                PackageManager::Paru => BTreeSet::new(),
                PackageManager::Apt => todo!(),
            };
            entry.insert(list);
        }
        Ok(self.package_groups.get(&manager).unwrap())
    }

    /// Get the list of packages a package *group* consists of.
    /// Not cached: this is only called for the few groups that actually show
    /// up in a configuration.
    pub fn packages_for_group(
        &mut self,
        manager: PackageManager,
        group: &str,
    ) -> Result<BTreeSet<String>> {
        match manager {
            PackageManager::Pacman => pacman::get_packages_for_group(group),
            PackageManager::Paru => Ok(BTreeSet::new()),
            PackageManager::Apt => todo!(),
        }
    }
}
