use std::{collections::HashSet, process::Command};

use anyhow::{Context, Result, bail};
use log::info;

use crate::{handlers::packages::PackageManager, system_state::SystemState};

/// Install a package via pacman.
/// We install packages in `--asexplicit` mode, so they show up as exiplictly installed packages.
/// Otherwise they wouldn't be detected by us if they already were installed as a dependency.
pub(super) fn install_packages(packages: Vec<String>) -> Result<()> {
    println!("Installing packages via pacman:");
    for name in &packages {
        println!("    - {name}");
    }

    let output = Command::new("pacman")
        .args(["--sync", "--refresh", "--noconfirm"])
        .args(packages)
        .output()
        .context("Failed to install pacman package {}")?;

    if !output.status.success() {
        bail!(
            "Failed to install pacman packages:\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    Ok(())
}

/// Uninstall a package via pacman.
/// Don't instruct pacman to create backup files, as all configuration is handled by bois.
/// Also recursively remove dependencies, we don't want to clutter the system with unneeded
/// dependencies. Any dependencies that're still needed should be explicitly required.
pub(super) fn uninstall_packages(
    system_state: &mut SystemState,
    mut names: Vec<String>,
) -> Result<()> {
    let explicit_packages = system_state.explicit_packages(PackageManager::Pacman)?;

    // Filter all packages that aren't explicitly installed.
    // TODO: This is not enough.
    //       In theory, we need to go through the whole dependency tree of each package that is
    //       to be removed and check if there are any dependents somewhere up the tree that're:
    //       - Explicitly installed
    //       - Not removed in **this** batch of packages.
    //
    //       As long as we don't do this, there's always a chance that a package cannot be
    //       uninstalled since another package still depends on it.
    //
    //       In the future, once we have this functionality, those packages can be marked as
    //       non-explicits (dependencies). They'll then no longer be tracked by bois and will be
    //       cleaned up whenever their dependants are uninstalled.
    names.retain(|name| explicit_packages.contains(name));

    if names.is_empty() {
        info!("No packages to uninstall");
        return Ok(());
    }

    println!("Uninstalling packages via pacman:");
    for name in &names {
        println!("    - {name}");
    }

    let output = Command::new("pacman")
        .args(["--remove", "--nosave", "--noconfirm"])
        .args(names)
        .output()
        .context("Failed to install pacman package {}")?;

    if !output.status.success() {
        bail!(
            "Failed to uninstall pacman packages:\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    // Update the installed packages cache.
    // Packages might be uninstalled when packages due to dependency removal.
    system_state.update_packages(PackageManager::Pacman)?;

    Ok(())
}
