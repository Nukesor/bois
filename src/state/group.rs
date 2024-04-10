use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context, Result};
use serde_derive::{Deserialize, Serialize};

use crate::{handlers::packages::PackageManager, helper::read_yaml};

use super::directory::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Group {
    /// The top-level configuration file for this group.
    pub config: GroupConfig,
    /// The content of this group's directory.
    pub directory: Directory,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GroupConfig {
    /// The content of this group's directory.
    #[serde(default)]
    pub defaults: GroupDefaults,
    /// All variables that're available during templating for this group.
    #[serde(default)]
    pub variables: HashMap<String, String>,
    /// Packages that should always be installed for this group.
    #[serde(default)]
    pub packages: HashMap<PackageManager, Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GroupDefaults {
    pub owner: Option<String>,
    pub group: Option<String>,
}

pub fn read_group(root: &Path, name: &str) -> Result<Group> {
    let group_dir = root.join(name);

    // Read the `group.yml` from the group directory.
    let Some(group_config) = read_yaml::<GroupConfig>(&group_dir, "group") else {
        bail!("Couldn't find group.yml in {group_dir:?}");
    };
    // Handle any deserialization issues of the group.yml
    let group_config = match group_config {
        Ok(group_config) => group_config,
        Err(err) => return Err(err).context("Failed to read group.yml in {group_dir:?}"),
    };

    // Recursively read all files in directory
    let mut directory = Directory::new(&group_dir);
    //let entries = std::fs::read_dir(&group_dir)
    //    .map_err(|err| Error::IoPathError(group_dir.clone(), "reading", err))?;
    //// Go through all entries in this directory
    //for entry in entries {
    //    let entry =
    //        entry.map_err(|err| Error::IoPathError(group_dir.clone(), "reading entry", err))?;

    //    read_file(root, &Path::new(""), entry, &mut files)?;
    //}

    Ok(Group {
        config: group_config,
        directory,
    })
}
