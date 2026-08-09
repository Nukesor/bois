use std::collections::BTreeMap;

use anyhow::Result;

use crate::{
    changeset::{PackageInstall, PackageUninstall},
    state::PackageManager,
    system_state::SystemState,
};

pub mod pacman;
pub mod paru;

/// Execute a list of package uninstalls.
///
/// This is done during the cleanup phase when packages have been removed since the last deploy.
/// Packages are grouped by package manager and then uninstalled in one go.
///
/// This must be done to prevent dependency issues when uninstalling groups of packages.
pub fn uninstall_packages(
    system_state: &mut SystemState,
    packages: &[PackageUninstall],
) -> Result<()> {
    let mut sorted_packages: BTreeMap<PackageManager, Vec<String>> = BTreeMap::new();

    // First up, sort all packages by manager.
    // That way, we get lists of packages that can be uninstalled in one go.
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

/// Execute a list of package installs.
///
/// This is done whenever a package is missing on the system.
/// Packages are grouped by package manager and then installed in one go.
pub fn install_packages(system_state: &mut SystemState, packages: &[PackageInstall]) -> Result<()> {
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

        // The install may have pulled in new packages/dependencies:
        // refresh the cached system view.
        system_state.update_packages(manager)?;
    }

    Ok(())
}
