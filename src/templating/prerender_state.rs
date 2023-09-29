use anyhow::{Context, Result};

use crate::{args::Arguments, config::Configuration, templating::file::File};

pub struct PrerenderState {
    pub config: Configuration,
    pub machine_name: String,
    pub files: Vec<File>,
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
            config: Configuration::default(),
            machine_name,
            files: Vec::new(),
        })
    }
}
