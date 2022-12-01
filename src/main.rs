use anyhow::{Context, Result};
use clap::Parser;

use cli::CliArguments;
use log::LevelFilter;
use pest::Parser as PestParser;
use pretty_env_logger::env_logger::Builder;

mod cli;
mod config;
mod error;
mod parser;

fn main() -> Result<()> {
    // Read any .env files
    dotenv::dotenv().ok();
    // Parse commandline options.
    let opt = CliArguments::parse();

    // Initalize everything
    init_app(opt.verbose)?;

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
