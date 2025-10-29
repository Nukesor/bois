use std::{collections::HashSet, process::Command};

use anyhow::{Context, Result, bail};
use log::info;

use crate::{handlers::packages::PackageManager, system_state::SystemState};

/// Install a package via paru.
pub(super) fn install_packages(packages: Vec<String>) -> Result<()> {
    println!("Installing packages via paru:");
    for name in &packages {
        println!("    - {name}");
    }

    // TODO: Check if there's a more elegant way of doing this.
    // See the book/src/system_configuration/package_manageres/paru.md section on the current
    // approach. Maybe make the aur user configurable.
    let output = Command::new("sudo")
        .args([
            "-u",
            "aur",
            "paru",
            "--sync",
            "--refresh",
            "--aur",
            "--noconfirm",
        ])
        .args(packages)
        .output()
        .context("Failed to install paru package {}")?;

    if !output.status.success() {
        bail!(
            "Failed to install paru packages:\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    Ok(())
}

/// Uninstall a package via paru.
/// Don't instruct paru to create backup files, as all configuration is handled by bois.
/// Also recursively remove dependencies, we don't want to clutter the system with unneeded
/// dependencies. Any dependencies that're still needed should be explicitly required.
pub(super) fn uninstall_packages(
    system_state: &mut SystemState,
    mut packages: Vec<String>,
) -> Result<()> {
    let explicit_packages = system_state.explicit_packages(PackageManager::Paru)?;

    // Filter all packages that aren't explicitly installed.
    // TODO: See respective todo in pacman handler.
    packages.retain(|name| explicit_packages.contains(name));

    if packages.is_empty() {
        info!("No packages to uninstall");
        return Ok(());
    }

    println!("Uninstalling packages...");

    let output = Command::new("pacman")
        .args(["--remove", "--nosave", "--noconfirm"])
        .args(packages)
        .output()
        .context("Failed to uninstall paru AUR package {}")?;

    if !output.status.success() {
        bail!(
            "Failed to uninstall paru packages:\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    Ok(())
}

/// Receive a list of **exlicitly** installed foreign (AUR) packages on the system.
/// Ignore packages that are installed as a dependency, as they might be removed at any point in
/// time when another package is uninstalled as a side-effect.
pub fn explicit_packages() -> Result<HashSet<String>> {
    // Get all explicitly installed packages
    let output = Command::new("pacman")
        .args(["--query", "--quiet", "--explicit", "--foreign"])
        .output()
        .context("Failed to read foreign pacman package list")?;

    if !output.status.success() {
        bail!(
            "Failed to get foreing pacman packages:\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    // Interpret the output as utf8 and split by lines.
    // Each package is on its own line.
    let packages =
        String::from_utf8(output.stdout).context("Couldn't deserialize pacman packages")?;

    Ok(packages.lines().map(ToOwned::to_owned).collect())
}
