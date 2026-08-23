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
pub struct HostConfig {
    /// Used to overwrite the target directory to which files should be deployed for
    /// this specific host. Must be an absolute path.
    // TODO(backwards compatibility): alias
    #[serde(default, alias = "target_directory")]
    pub target_dir: Option<PathBuf>,
    /// Default permissions that should be applied to all files and directories.
    // TODO(backwards compatibility): alias
    #[serde(default, alias = "file_defaults")]
    pub permission_defaults: HostDefaults,
    /// Cleaned settings on which components should be removed once they leave a host's
    /// configuration.
    #[serde(default)]
    pub cleanup: CleanupConfig,
    /// Traits that're required by this host.
    #[serde(default)]
    pub traits: Vec<String>,
    /// Packages that should always be installed for this host.
    #[serde(default)]
    pub packages: BTreeMap<PackageManager, BTreeSet<String>>,
    /// Services that should always be enabled for this host.
    #[serde(default)]
    pub services: BTreeMap<ServiceManager, BTreeSet<Service>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HostDefaults {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub file_mode: Option<u32>,
    pub directory_mode: Option<u32>,
}
