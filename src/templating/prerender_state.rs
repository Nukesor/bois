use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::args::Arguments;

pub struct PrerenderState {
    pub config: crate::config::Configuration,
    pub machine_name: String,
    pub root_dir: PathBuf,
    pub files: Vec<crate::templating::file::File>,
}

impl PrerenderState {
    pub fn new(args: &Arguments) -> Result<Self> {
        // Use the provided name or try to deduct it from the hostname.
        let machine_name = if let Some(name) = &args.name {
            name.clone()
        } else {
            hostname::get()
                .context("Couldn't determine the machine's name.")?
                .to_string_lossy()
                .to_string()
        };

        Ok(PrerenderState {
            config: crate::config::Configuration::default(),
            machine_name,
            root_dir: PathBuf::from("/home/nuke/.sysconfig"),
            files: Vec::new(),
        })
    }
}
