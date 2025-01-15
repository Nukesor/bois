use anyhow::{Result, bail};
use inquire::Confirm;
use log::trace;

use crate::{
    changeset::{Changeset, host_to_state, state_to_host, state_to_state},
    config::Configuration,
    handlers::{
        packages::{install_packages, uninstall_packages},
        paths::handle_path_operations,
    },
    state::State,
    system_state::SystemState,
    ui::{print_package_installs, print_package_uninstalls, print_path_changes},
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
    let previous_state = State::read_previous(&config)?;
    trace!("Previous state: {previous_state:#?}");

    // Create a new empty changeset.
    // All changes will be appended into this struct
    let mut changeset = Changeset::new();

    // ---------- Step 1: Detect system changes ----------
    // Create the changeset between the current system and the last deployment.
    // This will allows us to detect any changes that were done to the system,
    //
    // The user might have forgotten to integrate those changes into the bois config, so we
    // want to inform them about it.
    if let Some(previous_state) = &previous_state {
        let system_changes = host_to_state::create_changeset(
            &config,
            &mut system_state,
            previous_state,
            &desired_state,
        )?;

        // TODO: Logic to absorb system state
        if !system_changes.is_empty() {
            println!("Some untracked changes were detected on the system since last deployment.");
            if !system_changes.path_operations.is_empty() {
                print_path_changes(&system_changes.path_operations, &config)?;
            }

            if !dry_run {
                let ans = Confirm::new("These changes will be overwritten. Are you sure that's okay?")
                    .with_default(false)
                    .with_help_message("The changes above have been made on your system and haven't been merged into your config yet.")
                    .prompt();

                match ans {
                    Ok(true) => (),
                    _ => bail!("Aborting"),
                }
            }
        }
    };

    // ---------- Step 2: Detect old changes that need to be cleaned up ----------
    // Determine any cleanup that needs to be done due to changes in configuration since the
    // last deployment.
    if let Some(state) = &previous_state {
        let cleanup = state_to_state::create_changeset(&mut system_state, state, &desired_state)?;

        changeset.merge(cleanup);
    };

    // ---------- Step 3: Compute changes that will be deployed ----------
    // Create and execute the changeset to reach the desired state.
    let new_changes = state_to_host::create_changeset(&config, &desired_state, &mut system_state)?;

    changeset.merge(new_changes);

    // ------------------- Execution phase -------------------
    // We now start to actually execute commands.

    // ---------- Step 4: Uninstall unwanted packages ----------
    if !changeset.package_uninstalls.is_empty() {
        println!("Cleanup changes to be executed:");

        print_package_uninstalls(&changeset.package_uninstalls);
        println!();

        if !dry_run {
            uninstall_packages(&mut system_state, &changeset.package_uninstalls)?;
        } else {
            println!("Dry-run. Not uninstalling anything... yet");
        }
    }

    // ---------- Step 5: Install new packages ----------
    if !changeset.package_installs.is_empty() {
        // Print all package related changes .
        print_package_installs(&changeset.package_installs);
        println!();

        // Execute all package related changes.
        if !dry_run {
            install_packages(&changeset.package_installs)?;
        } else {
            println!("Dry-run. Not installing anything... yet");
        }
    }

    // ---------- Step 6: Execute all path operations ----------
    if !changeset.path_operations.is_empty() {
        print_path_changes(&changeset.path_operations, &config)?;
        println!();

        // Execute all path related changes.
        if !dry_run {
            handle_path_operations(&mut system_state, &changeset.path_operations)?;
        } else {
            println!("Dry-run. Not changing any files... yet");
        }
    }

    // Save the current desired state to disk for the next run.
    if !dry_run {
        desired_state.save()?;
    }

    Ok(())
}
