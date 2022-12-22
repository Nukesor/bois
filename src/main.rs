use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use args::Arguments;
use log::LevelFilter;
use pest::Parser as PestParser;
use pretty_env_logger::env_logger::Builder;

mod args;
mod config;
mod error;
mod file;
mod parser;

struct MetaConfig {
    config: config::Configuration,
    hostname: String,
    root_dir: PathBuf,
    files: Vec<file::File>,
}

impl MetaConfig {
    fn new() -> Result<Self> {
        let hostname = hostname::get()
            .context("Couldn't determine the machine's name.")?
            .to_string_lossy()
            .to_string();

        Ok(MetaConfig {
            config: (),
            hostname,
            root_dir: PathBuf::from("/home/nuke/.sys"),
            files: Vec::new(),
        })
    }
}

fn main() -> Result<()> {
    // Read any .env files
    dotenv::dotenv().ok();
    // Parse commandline options.
    let opt = Arguments::parse();

    // Initalize everything
    init_app(opt.verbose)?;

    let hostname = hostname::get().context("Couldn't determine the machine's name.")?;

    let text = r#"# bois_config_start
    # this_is_some_test:
    #    - "Geil"
    # bois_config_end
    test
    bois_config_end"#;

    let parsed = parser::ConfigParser::parse(parser::Rule::full_config, text)
        .context("Failed to parse query")?;
    dbg!(parsed);

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
