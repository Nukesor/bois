use std::{collections::HashSet, process::Command};

use anyhow::{bail, Context, Result};

use crate::changeset::PackageOperation;

pub fn remove_package(op: PackageOperation) -> Result<()> {
    let packages = get_installed_packages()?;

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
        String::from_utf8(output.stderr).context("Couldn't deserialize pacman packages")?;

    Ok(packages.lines().map(ToOwned::to_owned).collect())
}
