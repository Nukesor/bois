use std::{collections::HashSet, process::Command};

use anyhow::{Context, Result, bail};

/// Get the list of **exlicitly** installed foreign (AUR) packages on the system.
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
