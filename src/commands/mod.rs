use anyhow::Result;
use log::debug;

use crate::{CONFIG, args::Subcommand, config::bois::RawConfiguration};

mod deploy;
mod diff;
mod init;

pub fn run_subcommand(raw_config: RawConfiguration, subcommand: &Subcommand) -> Result<()> {
    // `init` must run before full configuration is built, as there's no config yet.
    if let Subcommand::Init { directory } = subcommand {
        return init::run_init(raw_config, directory);
    }

    // Build the final configuration based on the values from the config file.
    // All other values are populated with default values.
    let config = raw_config.build_configuration()?;
    debug!("Running with the following config:\n{config:#?}");

    // Set the config globally.
    // It's only ever accessed from inside minijinja template functions, which
    // cannot take extra arguments. See the docs on [CONFIG].
    CONFIG.set(config.clone()).unwrap();

    match subcommand {
        Subcommand::Plan => deploy::run_deploy(config, true),
        Subcommand::Deploy => deploy::run_deploy(config, false),
        Subcommand::Adopt => todo!(),
        Subcommand::Diff => diff::diff(config),
        Subcommand::Init { .. } => unreachable!("handled above"),
    }
}
