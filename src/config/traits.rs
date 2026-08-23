use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{
    config::{cleanup::CleanupConfig, services::Service},
    state::{PackageManager, ServiceManager},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TraitConfig {
    /// Used to overwrite the target directory to which files should be deployed for
    /// this specific trait. Must be an absolute path.
    // TODO(backwards compatibility): alias
    #[serde(default, alias = "target_directory")]
    pub target_dir: Option<PathBuf>,
    /// The content of this trait's directory.
    #[serde(default)]
    pub defaults: TraitDefaults,
    /// Cleaned settings on which components should be removed once they leave a
    /// trait's configuration.
    #[serde(default)]
    pub cleanup: CleanupConfig,
    /// Packages that should always be installed for this trait.
    #[serde(default)]
    pub packages: BTreeMap<PackageManager, BTreeSet<String>>,
    /// Services that should be enabled.
    #[serde(default)]
    pub services: BTreeMap<ServiceManager, BTreeSet<Service>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TraitDefaults {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub file_mode: Option<u32>,
    pub directory_mode: Option<u32>,
}
