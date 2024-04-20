use std::{collections::HashSet, process::Command};

use anyhow::{bail, Context, Result};

/// Install a package via pacman.
/// We install packages in `--asexplicit` mode, so they show up as exiplictly installed packages.
/// Otherwise they wouldn't be detected by us if they were installed as a dependency.
pub fn install_package(name: &str) -> Result<()> {
    println!("Installing package {name} via pacman");
    // TODO: Error handling
    let _output = Command::new("pacman")
        .args(["--sync", "--refresh", "--asexplicit", "--noconfirm", name])
        .output()
        .context("Failed to install pacman package {}")?;

    Ok(())
}

/// Uninstall a package via pacman.
/// Don't instruct pacman to create backup files, as all configuration is handled by bois.
/// Also recursively remove dependencies, we don't want to clutter the system with unneeded
/// dependencies. Any dependencies that're still needed should be explicitly required.
pub fn uninstall_package(name: &str) -> Result<()> {
    println!("Uninstalling package {name} via pacman");
    // TODO: Error handling
    let _output = Command::new("pacman")
        .args(["--remove", "--nosave", "--noconfirm", name])
        .output()
        .context("Failed to install pacman package {}")?;

    Ok(())
}

/// Receive a list of **exlicitly** installed packages on the system.
/// Ignore packages that are installed as a dependency, as they might be removed at any point in
/// time when another package is uninstalled as a side-effect.
pub fn get_installed_packages() -> Result<HashSet<String>> {
    // Get all explicitly installed packages
    let output = Command::new("pacman")
        .args(["--query", "--quiet", "--explicit"])
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
    let packages =
        String::from_utf8(output.stdout).context("Couldn't deserialize pacman packages")?;

    Ok(packages.lines().map(ToOwned::to_owned).collect())
}
