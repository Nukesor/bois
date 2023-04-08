use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use pretty_env_logger::env_logger::Builder;

use args::Arguments;
use file::discover_files;
use log::LevelFilter;

mod args;
mod config;
mod error;
mod file;
mod parser;

struct MetaConfig {
    config: config::Configuration,
    machine_name: String,
    root_dir: PathBuf,
    files: Vec<file::File>,
}

impl MetaConfig {
    fn new(args: &Arguments) -> Result<Self> {
        // Use the provided name or try to deduct it from the hostname.
        let machine_name = if let Some(name) = &args.name {
            name.clone()
        } else {
            hostname::get()
                .context("Couldn't determine the machine's name.")?
                .to_string_lossy()
                .to_string()
        };

        Ok(MetaConfig {
            config: config::Configuration::default(),
            machine_name,
            root_dir: PathBuf::from("/home/nuke/.sys"),
            files: Vec::new(),
        })
    }
}

fn main() -> Result<()> {
    // Read any .env files
    dotenv::dotenv().ok();
    // Parse commandline options.
    let args = Arguments::parse();

    // Initalize everything
    init_app(args.verbose)?;

    let config = MetaConfig::new(&args)?;

    discover_files(&config.root_dir, &PathBuf::from("./"))?;

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
