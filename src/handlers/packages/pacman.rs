use std::{collections::HashSet, process::Command};

use anyhow::{bail, Context, Result};
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

/// Receive a list of **all** installed packages on the system, including dependencies.
pub fn packages() -> Result<HashSet<String>> {
    let args = Vec::from(["--query", "--quiet", "--native"]);

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

/// Receive a list of **exlicitly** installed **native** (non-AUR) packages on the system.
/// Ignore packages that are installed as a dependency, as they might be removed at any point in
/// time when another package is uninstalled as a side-effect.
pub fn explicit_packages() -> Result<HashSet<String>> {
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
