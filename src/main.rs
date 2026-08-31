use std::io::IsTerminal;

use anyhow::Result;
use bois::{
    args::Arguments,
    commands::run_subcommand,
    config::bois::RawConfiguration,
    ui::theme::{self, Palette, Style, Theme},
};
use clap::Parser;
use log::LevelFilter;
use pretty_env_logger::env_logger::Builder;

fn main() -> Result<()> {
    // Read any .env files
    dotenv::dotenv().ok();
    // Parse commandline options.
    let args = Arguments::parse();

    // Initialize everything
    init_app(args.verbose)?;

    // Read the raw configuration file.
    // The full configuration is built later in `run_subcommand`.
    let raw_config = RawConfiguration::read(&args.config)?;

    run_subcommand(raw_config, &args.subcommand)?;

    Ok(())
}

/// Init better_panics.
/// Initialize logging.
/// Initialize the output style.
fn init_app(verbosity: u8) -> Result<()> {
    // Beautify panics for better debug output.
    better_panic::install();

    // Enable styled output on a stdout TTY.
    // TODO: `--color`/`--theme` flags and terminal background detection.
    theme::set(Style {
        enabled: std::io::stdout().is_terminal(),
        palette: Palette::preset(Theme::Dark),
    });

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
