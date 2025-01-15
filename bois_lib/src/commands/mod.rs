use anyhow::Result;

use crate::{args::Subcommand, config::Configuration};

mod deploy;
mod diff;
mod init;

pub fn run_subcommand(config: Configuration, subcommand: &Subcommand) -> Result<()> {
    match subcommand {
        Subcommand::Plan => deploy::run_deploy(config, true),
        Subcommand::Deploy => deploy::run_deploy(config, false),
        Subcommand::Absorb => todo!(),
        Subcommand::Init { directory } => init::run_init(config, directory),
        Subcommand::Diff => diff::diff(config),
    }
}
