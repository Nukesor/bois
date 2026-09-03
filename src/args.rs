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
    /// Take any configuration file or directory from the system and add it to the bois directory.
    ///
    /// If no trait is specified, the files are added to the current host by default.
    Adopt {
        /// The paths to the file or directory to add.
        paths: Vec<PathBuf>,

        /// `--base-dir` allows you to specify the parent directory under which those files should
        /// live inside the host/trait directory.
        #[clap(short, long)]
        base_dir: Option<String>,

        /// The trait to which the files should be added to.
        #[clap(short, long)]
        r#trait: Option<String>,
    },
    /// Setup a new bois directory.
    /// If no path is provided, it'll create the files inside of the current directory.
    Init {
        /// When provided, a new directory with that name will be created and used.
        directory: Option<PathBuf>,
    },
}
