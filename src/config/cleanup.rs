//! The `cleanup` configuration section, shared by host, group and directory
//! configuration files.

use serde::{Deserialize, Serialize};

/// Controls what bois cleans up once it leaves the desired state.
///
/// Files, packages and services are always cleaned up and not configurable
/// (for now); this only covers the resources where cleanup is opt-in.
///
/// The settings cascade: a host/group config sets the baseline for its whole
/// source tree, and a directory's `bois.yml` can override it (in either
/// direction) for its subtree. Unset fields inherit from the parent.
///
/// ```yaml
/// cleanup:
///   directories: true
/// ```
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CleanupConfig {
    /// Whether directories are removed (if empty) once they leave the config.
    pub directories: Option<bool>,
}
