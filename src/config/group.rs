use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{config::cleanup::CleanupConfig, state::PackageManager};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupConfig {
    /// Used to overwrite the target directory to which files should be deployed for
    /// this specific group. Must be an absolute path.
    #[serde(default)]
    pub target_directory: Option<PathBuf>,
    /// The content of this group's directory.
    #[serde(default)]
    pub defaults: GroupDefaults,
    /// Cleaned settings on which components should be removed once they leave a
    /// group's configuration.
    #[serde(default)]
    pub cleanup: CleanupConfig,
    /// Packages that should always be installed for this group.
    #[serde(default)]
    pub packages: BTreeMap<PackageManager, BTreeSet<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GroupDefaults {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub file_mode: Option<u32>,
    pub directory_mode: Option<u32>,
}
