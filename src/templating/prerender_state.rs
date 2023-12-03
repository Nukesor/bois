use anyhow::Result;

use crate::{args::Arguments, group_config::GroupConfig, templating::file::Entry};

/// This struct all configuration that's applicable for this machine.
/// This includes:
/// - All applicable groups
///     - Variables
///     - Directories
///     - Files/Templates
///     - In-file and in-directory configuration
pub struct PrerenderState {
    pub groups: Vec<Group>,
}

pub struct Group {
    /// The top-level configuration file for this group/host.
    group_config: GroupConfig,
    /// The list of all top-level files/directories.
    entries: Vec<Entry>,
}

impl PrerenderState {
    pub fn new(args: &Arguments) -> Result<Self> {
        Ok(PrerenderState { groups: Vec::new() })
    }
}
