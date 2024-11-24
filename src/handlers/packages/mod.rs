use std::collections::{BTreeMap, HashSet};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{
    changeset::{PackageInstall, PackageUninstall},
    system_state::SystemState,
};

pub mod pacman;
pub mod paru;

#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Display, Deserialize, Serialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "PascalCase")]
pub enum PackageManager {
    Pacman,
    Paru,
    Apt,
}

/// Return the set of all explicitly installed packages on the system.
pub fn get_detected_groups(manager: PackageManager) -> Result<HashSet<String>> {
    match manager {
        PackageManager::Pacman => pacman::detect_installed_groups(),
        PackageManager::Paru => Ok(HashSet::new()),
        PackageManager::Apt => todo!(),
    }
}

/// Execute a list of package uninstall.
///
/// This is done during the cleanup phase when packages have been removed since the last deploy.
/// Packages are grouped by package manager and then uninstalled in one go.
pub fn uninstall_packages(
    system_state: &mut SystemState,
    packages: &Vec<PackageUninstall>,
) -> Result<()> {
    let mut sorted_packages: BTreeMap<PackageManager, Vec<String>> = BTreeMap::new();

    // First up, sort all packages by manager.
    // That way, we get lists of packages that can be uninstalled in one go.
    //
    // This must be done to prevent dependency issues when uninstalling groups of packages.
    for pkg in packages {
        let list = sorted_packages.entry(pkg.manager).or_default();
        list.push(pkg.name.clone());
    }

    for (manager, packages) in sorted_packages {
        match manager {
            PackageManager::Pacman => pacman::uninstall_packages(system_state, packages)?,
            PackageManager::Paru => paru::uninstall_packages(system_state, packages)?,
            PackageManager::Apt => todo!(),
        }
    }

    Ok(())
}

/// Execute a list of package install.
///
/// This is done during whenever a package is missing on the system.
/// Packages are grouped by package manager and then installed in one go.
pub fn install_packages(packages: &Vec<PackageInstall>) -> Result<()> {
    let mut sorted_packages: BTreeMap<PackageManager, Vec<String>> = BTreeMap::new();

    // First up, sort all packages by manager.
    for pkg in packages {
        let list = sorted_packages.entry(pkg.manager).or_default();
        list.push(pkg.name.clone());
    }

    for (manager, packages) in sorted_packages {
        match manager {
            PackageManager::Pacman => pacman::install_packages(packages)?,
            PackageManager::Paru => paru::install_packages(packages)?,
            PackageManager::Apt => todo!(),
        }
    }

    Ok(())
}
