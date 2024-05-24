use anyhow::Result;

use crate::{args::Subcommand, config::Configuration};

pub mod deploy;

pub fn run_subcommand(config: Configuration, subcommand: &Subcommand) -> Result<()> {
    match subcommand {
        Subcommand::Plan => deploy::run_deploy(config, true),
        Subcommand::Deploy => deploy::run_deploy(config, false),
        Subcommand::Absorb => todo!(),
        Subcommand::Diff => todo!(),
    }
}
