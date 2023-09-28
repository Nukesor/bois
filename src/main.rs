use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use config::Configuration;
use pretty_env_logger::env_logger::Builder;

use args::Arguments;
use log::LevelFilter;
use templating::discover_files;

mod args;
mod config;
mod error;
mod parser;
mod templating;

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
    if found_config {
        config.save(&args.config)?;
    }

    let prerender_state = templating::prerender_state::PrerenderState::new(&args)?;

    discover_files(&prerender_state.root_dir, &PathBuf::from("./"))?;

    Ok(())
}

/// Init better_panics
/// Initialize logging
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
