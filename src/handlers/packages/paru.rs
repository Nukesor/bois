use std::{collections::HashSet, process::Command};

use anyhow::{bail, Context, Result};

/// Install a package via paru.
pub fn install_package(name: &str) -> Result<()> {
    println!("Installing package {name} via paru");
    // TODO: Check if there's a more elegant way of doing this.
    //       See the docs/AUR.md section on the current approach.
    let output = Command::new("sudo")
        .args([
            "-u",
            "aur",
            "paru",
            "--sync",
            "--refresh",
            "--aur",
            "--noconfirm",
            name,
        ])
        .output()
        .context("Failed to install paru package {}")?;

    if !output.status.success() {
        bail!(
            "Failed to uninstall paru package:\n{name}:\nStdout: {}\nStderr: {}",
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
pub fn uninstall_package(name: &str) -> Result<()> {
    println!("Uninstalling package {name} via paru");

    let output = Command::new("pacman")
        .args(["--remove", "--nosave", "--noconfirm", name])
        .output()
        .context("Failed to uninstall paru AUR package {}")?;

    if !output.status.success() {
        bail!(
            "Failed to uninstall paru package:\n{name}:\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    Ok(())
}

/// Receive a list of **exlicitly** installed packages on the system.
/// Ignore packages that are installed as a dependency, as they might be removed at any point in
/// time when another package is uninstalled as a side-effect.
pub fn explicit_packages() -> Result<HashSet<String>> {
    // Get all explicitly installed packages
    let output = Command::new("pacman")
        .args(["--query", "--quiet", "--foreign"])
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
