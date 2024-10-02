// Allow dead code while prototyping.
#![allow(clippy::assigning_clones)]
// Allow dead code while prototyping.
#![allow(dead_code)]
use std::sync::OnceLock;

use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use pretty_env_logger::env_logger::Builder;

mod args;
mod changeset;
mod commands;
mod config;
mod constants;
mod error;
mod handlers;
mod helper;
mod password_managers;
mod state;
mod system_state;
mod templating;
mod ui;

use args::Arguments;
use commands::run_subcommand;
use config::Configuration;

/// Expose the config as a global.
/// This is somewhat of an antipatter, but is needed to access the configuration inside of
/// minijinja custom filters/functions. We have no way to pass additional arguments to those, as
/// they're called by minijinja. (We could maybe use closures when registering the minijinja
/// functions, but that feels also somewhat ugly.)
///
/// Avoid to use this anywhere outside of minijinja's filters/functions.
static CONFIG: OnceLock<Configuration> = OnceLock::new();

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

    // Set the config globally.
    CONFIG.set(config.clone()).unwrap();

    run_subcommand(config, &args.subcommand)?;

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
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    Builder::new().filter_level(level).init();

    Ok(())
}
