// Allow dead code while prototyping.
#![allow(clippy::assigning_clones)]
// Allow dead code while prototyping.
#![allow(dead_code)]

use anyhow::Result;
use bois::{CONFIG, args::Arguments, commands::run_subcommand, config::bois::RawConfiguration};
use clap::Parser;
use log::{LevelFilter, debug};
use pretty_env_logger::env_logger::Builder;

fn main() -> Result<()> {
    // Read any .env files
    dotenv::dotenv().ok();
    // Parse commandline options.
    let args = Arguments::parse();

    // Initalize everything
    init_app(args.verbose)?;

    // Build the final configuration based on the values from the config file.
    // All other values are populated with default values.
    let raw_config = RawConfiguration::read(&args.config)?;
    let config = raw_config.build_configuration()?;

    debug!("Running with the following config:\n{config:#?}");

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
