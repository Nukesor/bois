use anyhow::Result;

use crate::{args::Arguments, templating::file::File};

pub struct PrerenderState {
    pub files: Vec<File>,
}

impl PrerenderState {
    pub fn new(args: &Arguments) -> Result<Self> {
        Ok(PrerenderState { files: Vec::new() })
    }
}
