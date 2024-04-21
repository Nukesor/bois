use anyhow::Result;
use clap::Parser;
use config::Configuration;
use pretty_env_logger::env_logger::Builder;

mod args;
mod changeset;
mod config;
mod error;
mod handlers;
mod helper;
mod state;
mod system_state;

use args::Arguments;
use log::{debug, info, LevelFilter};

use crate::{changeset::state_to_host, handlers::handle_changeset, system_state::SystemState};

fn main() -> Result<()> {
    // Read any .env files
    dotenv::dotenv().ok();
    // Parse commandline options.
    let args = Arguments::parse();

    // Initalize everything
    init_app(args.verbose)?;

    let (config, found_config) = Configuration::read(&args.config)?;
    // In case we didn't find a configuration file, write a default configuration file
    // to given path or to the default configuration path.
    if !found_config {
        config.save(&args.config)?;
    }

    // This struct will hold state of the current system to compare it with the desired state.
    // This doesn't contain all system state, only stuff like packages and system services.
    // It's basically a cache struct, so we don't repeatedly run the same queries all the time.
    let mut system_state = SystemState::new()?;

    // Read the current desired system state from the files in the specified bois directory.
    let desired_state = state::State::new(config)?;
    debug!("Config state: {desired_state:#?}");
    // Run some basic checks on the read state.
    desired_state.lint();

    // Read the state of the previous run, if existant.
    // This state will be used to determine:
    // - Any changes on the system's files since the last deployment
    // - Cleanup work that might need to be done for the new desired state.
    let previous_state = state::State::read_previous()?;
    if let Some(previous_state) = previous_state {
        // First, we create a changeset between the last desired state at the point of the last deployment
        // and the current system statement.
        // This will allows us to detect any changes that were done to the system.
        // The changes done to the system will basically be the reverted actions of the changeset.
        let changeset = state_to_host::create_changeset(&previous_state, &mut system_state)?;
        if !changeset.is_empty() {
            info!("Some untracked changes were detected on the system since last deployment.");
            for change in changeset {
                println!("Change (reverted): {change:?}");
            }
        }
    }

    // Create and execute the changeset to reach the actual desired state.
    {
        let changeset = state_to_host::create_changeset(&desired_state, &mut system_state)?;
        println!("Changeset: {changeset:#?}");
        handle_changeset(changeset)?;
    }

    // Save the current desired state to disk for the next run.
    desired_state.save()?;

    Ok(())
}

/// Init better_panics.
/// Initialize logging.
fn init_app(verbosity: u8) -> Result<()> {
    // Beautify panics for better debug output.
    better_panic::install();

    // Set the verbosity level and initialize the logger.
    let level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    Builder::new().filter_level(level).init();

    Ok(())
}
