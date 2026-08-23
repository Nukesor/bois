use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::{
    changeset::{ServiceDisable, ServiceEnable},
    config::bois::RunMode,
    state::ServiceManager,
    system_state::SystemState,
};

/// Enable a list of systemd units, one unit at a time.
///
/// Each unit is printed before its `systemctl` call runs, so if a call fails
/// it's immediately clear which unit is responsible.
///
/// Units with the `start` flag are enabled via `--now`, which also starts
/// them in the same call.
pub(super) fn enable_services(
    system_state: &mut SystemState,
    services: &[&ServiceEnable],
) -> Result<()> {
    println!("Enabling services via systemd:");
    for service in services {
        if service.start {
            println!("    - {} (starting immediately)", service.name);
            systemctl(system_state.mode(), &["enable", "--now"], &service.name)?;
        } else {
            println!("    - {}", service.name);
            systemctl(system_state.mode(), &["enable"], &service.name)?;
        }

        // Update the service state cache per unit, so successfully handled
        // units are reflected even if a later unit fails.
        system_state.update_service_enabled(ServiceManager::Systemd, &service.name, true);
        if service.start {
            system_state.update_service_active(ServiceManager::Systemd, &service.name, true);
        }
    }

    Ok(())
}

/// Stop and disable a list of systemd units, one unit at a time.
///
/// Each unit is printed before its `systemctl` call runs, so if a call fails
/// it's immediately clear which unit is responsible.
pub(super) fn disable_services(
    system_state: &mut SystemState,
    services: &[&ServiceDisable],
) -> Result<()> {
    println!("Stopping and disabling services via systemd:");
    for service in services {
        println!("    - {}", service.name);
        // `--now` also stops the unit in the same call.
        systemctl(system_state.mode(), &["disable", "--now"], &service.name)?;

        // Update the service state cache per unit, so successfully handled
        // units are reflected even if a later unit fails.
        system_state.update_service_enabled(ServiceManager::Systemd, &service.name, false);
        system_state.update_service_active(ServiceManager::Systemd, &service.name, false);
    }

    Ok(())
}

/// Run a systemctl subcommand on a single unit.
///
/// In user mode, the user's own systemd instance is targeted via `--user`.
fn systemctl(mode: RunMode, args: &[&str], unit: &str) -> Result<()> {
    let mut full_args: Vec<&str> = Vec::new();
    if mode == RunMode::User {
        full_args.push("--user");
    }
    full_args.extend(args);
    full_args.push(unit);

    let output = Command::new("systemctl")
        .args(&full_args)
        .output()
        .context("Failed to run systemctl")?;

    if !output.status.success() {
        bail!(
            "Failed to run `systemctl {}`:\nStdout: {}\nStderr: {}",
            full_args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    Ok(())
}
