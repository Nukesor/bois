use anyhow::Result;
use log::trace;

use crate::{
    changeset::{host_to_state, state_to_host, state_to_state, Change},
    config::Configuration,
    handlers::handle_changeset,
    state::State,
    system_state::SystemState,
    ui::{print_package_additions, print_package_removals, print_path_changes},
};

pub fn run_deploy(config: Configuration, dry_run: bool) -> Result<()> {
    // This struct will hold state of the current system to compare it with the desired state.
    // This doesn't contain all system state, only stuff like packages and system services.
    // It's basically a cache struct, so we don't repeatedly run the same queries all the time.
    let mut system_state = SystemState::new()?;
    trace!("System state: {system_state:#?}");

    // Read the current desired system state from the files in the specified bois directory.
    let desired_state = State::new(&config, &mut system_state)?;
    trace!("Config state: {desired_state:#?}");

    // Read the state of the previous run, if existing.
    // This state will be used to determine:
    // - Any changes on the system's files since the last deployment
    // - Cleanup work that might need to be done for the new desired state.
    let previous_state = State::read_previous()?;
    trace!("Previous state: {previous_state:#?}");

    // ---------- Step 1: Detect system changes ----------
    // Create the changeset between the current system and the last deployment.
    // This will allows us to detect any changes that were done to the system,
    //
    // The user might have forgotten to integrate those changes into the bois config, so we
    // want to inform them about it.
    let system_changes = match &previous_state {
        Some(state) => host_to_state::create_changeset(&mut system_state, state, &desired_state)?,
        None => None,
    };

    // ---------- Step 2: Detect old changes that need to be cleaned up ----------
    // Determine any cleanup that needs to be done due to changes in configuration since the
    // last deployment.
    let cleanup_changes = match &previous_state {
        Some(state) => state_to_state::create_changeset(&mut system_state, state, &desired_state)?,
        None => None,
    };

    // ---------- Step 3: Compute changes that will be deployed ----------
    // Create and execute the changeset to reach the desired state.
    let new_changes = state_to_host::create_changeset(&config, &desired_state, &mut system_state)?;

    // ------------------- Execution phase -------------------
    // We now start to actually execute commands.

    // ---------- Step 4: Ask whether system changes should be absorbed. ----------
    // TODO: Logic to absorb system state
    if let Some(changes) = system_changes {
        println!("Some untracked changes were detected on the system since last deployment.");
        for change in changes {
            println!("  {change:?}");
        }
    }

    // ---------- Step 5: Execute cleanup tasks ----------
    if let Some(changes) = cleanup_changes {
        println!("Cleanup changes to be executed:");
        // Filter all package related changes.
        let (package_changes, _rest): (Vec<_>, Vec<_>) = changes
            .into_iter()
            .partition(|change| matches!(change, Change::PackageChange(_)));

        // Print all package related changes .
        if !package_changes.is_empty() {
            print_package_removals(&package_changes);

            println!();
        }

        if !dry_run {
            handle_changeset(&mut system_state, &package_changes)?;
        }
    }

    // ---------- Step 6: Execute actual new deployment tasks ----------
    if let Some(changes) = new_changes {
        // Filter all package related changes.
        let (package_changes, rest): (Vec<_>, Vec<_>) = changes
            .into_iter()
            .partition(|change| matches!(change, Change::PackageChange(_)));

        // Print all package related changes .
        if !package_changes.is_empty() {
            print_package_additions(&package_changes);

            println!();
        }

        // Print all file related changes.
        let (path_changes, _service_changes): (Vec<_>, Vec<_>) = rest
            .into_iter()
            .partition(|change| matches!(change, Change::PathChange(_)));

        if !path_changes.is_empty() {
            print_path_changes(&path_changes)?;
            println!();
        }

        // Execute all package related changes.
        if !dry_run {
            handle_changeset(&mut system_state, &package_changes)?;
        }

        // Execute all path related changes.
        if !dry_run {
            handle_changeset(&mut system_state, &path_changes)?;
        }
    }

    // Save the current desired state to disk for the next run.
    if !dry_run {
        desired_state.save()?;
    }

    Ok(())
}
