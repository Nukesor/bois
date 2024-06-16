use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::{error::Error, handlers::packages::PackageManager, helper::read_yaml};

use super::{directory::*, file::read_entry};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Group {
    /// The name of this group
    pub name: String,
    /// The top-level configuration file for this group.
    pub config: GroupConfig,
    /// The content of this group's directory.
    pub directory: Directory,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupConfig {
    /// The content of this group's directory.
    #[serde(default)]
    pub defaults: GroupDefaults,
    /// All variables that're available during templating for this group.
    #[serde(default)]
    pub variables: HashMap<String, String>,
    /// Packages that should always be installed for this group.
    #[serde(default)]
    pub packages: HashMap<PackageManager, HashSet<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GroupDefaults {
    pub owner: Option<String>,
    pub group: Option<String>,
}

pub fn read_group(root: &Path, name: &str) -> Result<Group> {
    let group_dir = root.join("groups").join(name);

    // Read the `group.yml` from the group directory.
    let config = read_yaml::<GroupConfig>(&group_dir, "group")?;

    // Recursively read all files in directory
    let mut directory = Directory::new(&group_dir);
    let entries = std::fs::read_dir(&group_dir)
        .map_err(|err| Error::IoPath(group_dir.clone(), "reading", err))?;

    // Go through all entries in this directory
    for entry in entries {
        let entry = entry.map_err(|err| Error::IoPath(group_dir.clone(), "reading entry", err))?;

        // Don't include the group configuration file. It's already handled above
        if entry.file_name() == "group.yml" {
            continue;
        }

        read_entry(&group_dir, Path::new(""), entry, &mut directory, None)?;
    }

    Ok(Group {
        name: name.to_string(),
        config,
        directory,
    })
}
