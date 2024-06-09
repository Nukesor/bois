// Allow dead code while prototyping.
#![allow(dead_code)]
use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use pretty_env_logger::env_logger::Builder;

mod args;
mod changeset;
mod commands;
mod config;
mod error;
mod handlers;
mod helper;
mod state;
mod system_state;

use args::Arguments;
use commands::run_subcommand;
use config::Configuration;

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
