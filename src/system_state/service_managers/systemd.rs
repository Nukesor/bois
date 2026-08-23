use std::process::Command;

use anyhow::{Context, Result};

use crate::config::bois::RunMode;

/// The systemd unit file states that need no enablement by bois.
///
/// - `enabled`/`enabled-runtime`: the unit is already enabled.
/// - `static`/`indirect`/`generated`/`transient`/`alias`: unit types cannot be handled by us, so
///   there's nothing for us to do.
const ENABLED_STATES: [&str; 7] = [
    "enabled",
    "enabled-runtime",
    "static",
    "indirect",
    "generated",
    "transient",
    "alias",
];

/// Check whether a systemd unit counts as enabled.
///
/// A unit that doesn't (yet) exist counts as "not enabled". This may happen when a unit's
/// unit file will be installed/deployed in the very same run.
pub(super) fn is_enabled(name: &str, mode: RunMode) -> Result<bool> {
    let output = systemctl_command(mode)
        .args(["is-enabled", name])
        .output()
        .context(format!("Failed to query enabled state of unit {name}"))?;

    // `is-enabled` prints the unit file state on the first stdout line.
    // For unknown units, stdout is empty and an error is printed to stderr,
    // which correctly results in "not enabled".
    let stdout = String::from_utf8_lossy(&output.stdout);
    let state = stdout.lines().next().unwrap_or_default().trim();

    Ok(ENABLED_STATES.contains(&state))
}

/// Check whether a systemd unit is currently active.
///
/// Unknown units simply count as inactive.
pub(super) fn is_active(name: &str, mode: RunMode) -> Result<bool> {
    let output = systemctl_command(mode)
        .args(["is-active", "--quiet", name])
        .output()
        .context(format!("Failed to query active state of unit {name}"))?;

    Ok(output.status.success())
}

/// Build a `systemctl` invocation for the given mode.
fn systemctl_command(mode: RunMode) -> Command {
    let mut command = Command::new("systemctl");
    if mode == RunMode::User {
        command.arg("--user");
    }
    command
}
