use anyhow::Result;
use log::{info, trace};

use crate::{
    changeset::{state_to_host, state_to_state, ChangeSet},
    config::Configuration,
    handlers::handle_changeset,
    state::State,
    system_state::SystemState,
};

pub fn run_deploy(config: Configuration, dry_run: bool) -> Result<()> {
    // This struct will hold state of the current system to compare it with the desired state.
    // This doesn't contain all system state, only stuff like packages and system services.
    // It's basically a cache struct, so we don't repeatedly run the same queries all the time.
    let mut system_state = SystemState::new()?;

    // Read the current desired system state from the files in the specified bois directory.
    let desired_state = State::new(config)?;
    trace!("Config state: {desired_state:#?}");
    // Run some basic checks on the read state.
    desired_state.lint();

    // Read the state of the previous run, if existant.
    // This state will be used to determine:
    // - Any changes on the system's files since the last deployment
    // - Cleanup work that might need to be done for the new desired state.
    let previous_state = State::read_previous()?;

    // ---------- Step 1: Detect system changes ----------
    // Create the changeset between the current system and the last deployment.
    // This will allows us to detect any changes that were done to the system,
    // The changes done to the system will basically be the reverted actions of the changeset.
    let system_changes = match &previous_state {
        Some(state) => state_to_host::create_changeset(&state, &mut system_state)?,
        None => None,
    };

    // ---------- Step 2: Detect changes that need cleanup ----------
    // Determine any cleanup that needs to be done due to changes in configuration since the
    // last deployment.
    let cleanup_changes = match &previous_state {
        Some(state) => state_to_state::create_changeset(state, &desired_state),
        None => None,
    };

    // ---------- Step 3: Detect changes that need cleanup ----------
    // Create and execute the changeset to reach the actual desired state.
    let new_changes = state_to_host::create_changeset(&desired_state, &mut system_state)?;

    // ------------------- Execution phase -------------------
    // We now start to actually execute commands.
    // Save the current desired state to disk for the next run.

    // ---------- Step 4: Ask whether system changes should be absorbed. ----------
    // TODO: Logic to absorb system state
    if let Some(changes) = system_changes {
        println!("Some untracked changes were detected on the system since last deployment.");
        for change in changes {
            println!("  Change (reverted): {change:?}");
        }
    }

    // ---------- Step 5: Execute cleanup tasks ----------
    if let Some(changes) = cleanup_changes {
        println!("Cleanup changes to be executed:");
        for change in changes.iter() {
            println!("  {change:?}");
        }
        if !dry_run {
            handle_changeset(&changes)?;
        }
    }

    // ---------- Step 6: Execute actual new deployment tasks ----------
    if let Some(changes) = new_changes {
        println!("New changes that need to be deployed:");
        for change in changes.iter() {
            println!(" {change:?}");
        }
        if !dry_run {
            handle_changeset(&changes)?;
        }
    }

    desired_state.save()?;

    Ok(())
}
