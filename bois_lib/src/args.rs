use std::path::PathBuf;

use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[clap(
    name = "bois",
    about = "A configuration management tool for your system or user dotfiles.",
    author,
    version
)]
pub struct Arguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// The path to the configuration file that should be used.
    #[clap(short, long)]
    pub config: Option<PathBuf>,

    /// The name of the machine.
    /// This is usually automatically deducted via the hostname.
    #[clap(short, long)]
    pub name: Option<String>,

    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Parser, Debug)]
pub enum Subcommand {
    /// Run a dry-run on the current system and see all changes that would executed.
    Plan,
    /// Actually deploy all changes to the system
    Deploy,
    /// Show the diff between the current system and the target.
    /// This only shows differences in system services and packages.
    Diff,
    /// Check the system for any changes since the last deployment.
    /// If any are found, try to integrate them back into the configuration.
    Absorb,
    /// Setup a new bois directory.
    /// If no name is given, it'll create the files inside of the current directory.
    Init {
        /// When provided, a new directory with that name will be created and used.
        directory: Option<PathBuf>,
    },
}
