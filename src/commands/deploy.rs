use anyhow::Result;

use crate::{
    aggregators::aggregate_state,
    changeset::{
        Changeset,
        cleanup::post_cleanup_state,
        cleanup_changeset,
        deploy_changeset,
        detect_drift,
    },
    config::bois::Configuration,
    handlers::{
        packages::{install_packages, uninstall_packages},
        paths::execute_path_operations,
        services::{disable_services, enable_services},
    },
    state::State,
    system_state::SystemState,
    ui::{
        stages::{cleanup::handle_cleanup, deploy::handle_deploy, drift::handle_drift},
        style::Status,
    },
};

/// Perform a full run.
///
/// TODO: dev_docs are completely outdated.
/// The overall flow, as documented in `dev_docs/Architecture/Stages.md`:
///
/// 1. Report drift on the system since the last deploy and ask for confirmation before it gets
///    overwritten.
/// 2. Compute + execute the cleanup changeset, then persist an intermediate state.
/// 3. Compute + execute the deploy changeset.
/// 4. Persist final deployed state for the next run.
pub fn run_deploy(config: Configuration, dry_run: bool) -> Result<()> {
    // Gather all cacheable system info that we may need during this run.
    let mut system_state = SystemState::new(config.mode)?;

    // Read the desired system state from the files in the bois directory.
    let desired_state = aggregate_state(&config, &mut system_state)?;

    // Read the state of the previous run, if any. This is used to determine:
    // - Any changes on the system's files since the last run.
    // - Cleanup work that's needed for the desired state.
    let previous_state = State::read_previous(&config)?;

    // ---------- Step 1: Detect drift ----------
    // Compare the last deployed state against the system. The user might
    // have forgotten to integrate manual changes into the bois config, so we
    // inform them before anything gets overwritten.
    let mut drift_exists = false;
    if let Some(previous) = &previous_state {
        let drift = detect_drift(previous, &desired_state, &mut system_state)?;
        drift_exists = !drift.is_empty();

        if drift_exists {
            handle_drift(&drift, &config, dry_run)?;
        }
    }

    // ---------- Step 2: Cleanup ----------
    // Determine everything the previous run left behind that's no
    // longer part of the desired state and remove it.
    let cleanup = match &previous_state {
        Some(previous) => cleanup_changeset(previous, &desired_state, &mut system_state)?,
        None => Changeset::new(),
    };

    let cleanup_exists = !cleanup.is_empty();
    if cleanup_exists {
        if drift_exists {
            println!();
        }
        handle_cleanup(&cleanup);

        if !dry_run {
            // Cleanup in the following order:
            // - System services.
            // - On-disk files.
            // - Packages.
            if !cleanup.service_disables.is_empty() {
                disable_services(&mut system_state, &cleanup.service_disables)?;
                println!("{} Services disabled", Status::Applied.styled());
            }
            if !cleanup.path_cleanup.is_empty() {
                execute_path_operations(&cleanup.path_cleanup)?;
                println!("{} Files removed", Status::Applied.styled());
            }
            if !cleanup.package_uninstalls.is_empty() {
                uninstall_packages(&mut system_state, &cleanup.package_uninstalls)?;
                println!("{} Packages uninstalled", Status::Applied.styled());
            }

            // Persist the left-over state of the last run after cleanup.
            // If the following deploy phase aborts, the next run's drift detection won't blame
            // the intentional cleanup on the user.
            if let Some(previous) = &previous_state {
                post_cleanup_state(previous, &cleanup).save()?;
            }
        }
    }

    // ---------- Step 3: Deploy ----------
    // Create the changeset that transforms the system into the desired state.
    let deploy = deploy_changeset(&desired_state, &mut system_state)?;

    if deploy.is_empty() && cleanup.is_empty() {
        println!("Everything is up to date.");
        if !dry_run {
            desired_state.save()?;
        }
        return Ok(());
    }

    if !deploy.is_empty() {
        if drift_exists || cleanup_exists {
            println!();
        }

        handle_deploy(&deploy)?;
    }

    if dry_run {
        println!("Dry-run. Not doing anything... yet");
        return Ok(());
    }

    // Deploy order
    // - Packages as they may install/create:
    //  - directories that we want to deploy to
    //  - systemd service files
    // - Files.
    // - Services.
    //  - Deployed last, as they may require unit files that're installed by packages or us.
    //
    // TODO: Check what happens if a path is scheduled for creation, but that path is then
    // created by a package installation
    if !deploy.package_installs.is_empty() {
        install_packages(&mut system_state, &deploy.package_installs)?;
        println!("{} Packages installed", Status::Applied.styled());
    }
    if !deploy.path_operations.is_empty() {
        execute_path_operations(&deploy.path_operations)?;
        println!("{} Files deployed", Status::Applied.styled());
    }
    if !deploy.service_enables.is_empty() {
        enable_services(&mut system_state, &deploy.service_enables)?;
        println!("{} Services enaabled", Status::Applied.styled());
    }

    // ---------- Step 4: Persist the new state ----------
    desired_state.save()?;

    Ok(())
}
