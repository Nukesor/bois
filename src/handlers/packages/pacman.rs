use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::changeset::PackageOperation;

pub fn remove_package(op: PackageOperation) -> Result<()> {
    let packages = get_installed_packages()?;

    Ok(())
}

pub fn get_installed_packages() -> Result<Vec<String>> {
    // Get an up-to-date list of all pacman packages.
    let output = Command::new("pacman")
        .args(["-S", "-y", "-s"])
        .output()
        .context("Failed to read pacman packages list")?;

    if !output.status.success() {
        bail!(
            "Failed to get pacman package list:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let packages =
        String::from_utf8(output.stderr).context("Couldn't deserialize pacman packages")?;

    Ok(packages.lines().map(ToOwned::to_owned).collect())
}
