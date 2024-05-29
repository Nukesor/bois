use std::{collections::HashSet, process::Command};

use anyhow::{bail, Context, Result};
use log::{debug, info};

use crate::{handlers::packages::PackageManager, system_state::SystemState};

/// Install a package via pacman.
/// We install packages in `--asexplicit` mode, so they show up as exiplictly installed packages.
/// Otherwise they wouldn't be detected by us if they already were installed as a dependency.
pub(super) fn install_package(name: &str) -> Result<()> {
    debug!("Installing package {name} via pacman");
    let output = Command::new("pacman")
        .args(["--sync", "--refresh", "--noconfirm", "--asexplicit", name])
        .output()
        .context("Failed to install pacman package {}")?;

    if !output.status.success() {
        bail!(
            "Failed to install pacman package:\n{name}:\nStdout: {}\nStderr: {}",
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
pub(super) fn uninstall_package(system_state: &mut SystemState, name: &str) -> Result<()> {
    debug!("Uninstalling package {name} via pacman");
    let installed_packages = system_state.installed_packages(PackageManager::Pacman)?;
    if !installed_packages.contains(name) {
        info!("Package {name} is already uninstalled.");
        return Ok(());
    }

    let output = Command::new("pacman")
        .args(["--remove", "--nosave", "--noconfirm", name])
        .output()
        .context("Failed to install pacman package {}")?;

    if !output.status.success() {
        bail!(
            "Failed to uninstall pacman package:\n{name}:\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    // Update the installed packages cache.
    // Packages might be uninstalled when packages due to dependency removal.
    system_state.update_packages(PackageManager::Pacman)?;

    Ok(())
}

/// Receive a list of **exlicitly** installed packages on the system.
/// Ignore packages that are installed as a dependency, as they might be removed at any point in
/// time when another package is uninstalled as a side-effect.
pub fn get_installed_packages() -> Result<HashSet<String>> {
    let args = Vec::from(["--query", "--quiet", "--explicit", "--native"]);

    // Get all explicitly installed packages
    let output = Command::new("pacman")
        .args(args)
        .output()
        .context("Failed to read pacman packages list")?;

    if !output.status.success() {
        bail!(
            "Failed to query installed pacman packages:\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    // Interpret the output as utf8 and split by lines.
    // Each package is on its own line.
    let packages =
        String::from_utf8(output.stdout).context("Couldn't parse pacman output as utf-8")?;

    let packages: HashSet<String> = packages.lines().map(ToOwned::to_owned).collect();

    Ok(packages)
}

/// Query the list of all packages that belong to a specific group.
pub fn get_packages_for_group(name: &str) -> Result<HashSet<String>> {
    let output = Command::new("pacman")
        .args(["--sync", "--groups", name])
        .output()
        .context(format!("Failed to get pacman packages for group {name}"))?;

    if !output.status.success() {
        bail!(
            "Failed to get pacman packages for group {name}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Interpret the output as utf8 and split by lines.
    // Each package is on its own line.
    let group_package_tuples =
        String::from_utf8(output.stdout).context("Couldn't parse pacman output as utf-8")?;

    // Split the tuples into the groups.
    // duplicate lines are implicitly removed due to the HashSet.
    let packages: HashSet<String> = group_package_tuples
        .lines()
        .map(|line| line.split(' ').collect::<Vec<&str>>()[1].to_owned())
        .collect();

    Ok(packages)
}

/// Get the list of packages that belong to a group.
/// That way we detect, whether any of the packages in the bois config are actually groups.
///
/// If so, we need to do a bit of special handling in the
///
/// The `pacman -Qm` package lists all packages that belong to a group in a
/// `group-name package-name` tuple per line.
pub fn detect_installed_groups() -> Result<HashSet<String>> {
    let output = Command::new("pacman")
        .args(["--query", "--groups", "--quiet"])
        .output()
        .context("Failed to read pacman packages list")?;

    if !output.status.success() {
        bail!(
            "Failed to get pacman package list:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Interpret the output as utf8 and split by lines.
    // Each package is on its own line.
    let group_package_tuples =
        String::from_utf8(output.stdout).context("Couldn't parse pacman output as utf-8")?;

    // Split the tuples into the groups.
    // duplicate lines are implicitly removed due to the HashSet.
    let groups: HashSet<String> = group_package_tuples
        .lines()
        .map(|line| line.split(' ').next().unwrap().to_owned())
        .collect();

    Ok(groups)
}
