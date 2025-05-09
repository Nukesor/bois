use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

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
    /// Used to overwrite the target directory to which files should be deployed for
    /// this specific group.
    #[serde(default)]
    pub target_directory: Option<PathBuf>,
    /// The content of this group's directory.
    #[serde(default)]
    pub defaults: GroupDefaults,
    /// Packages that should always be installed for this group.
    #[serde(default)]
    pub packages: HashMap<PackageManager, HashSet<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GroupDefaults {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub file_permissions: Option<u32>,
    pub directory_permissions: Option<u32>,
}

pub fn read_group(root: &Path, name: &str, template_vars: &serde_yaml::Value) -> Result<Group> {
    let group_dir = root.join("groups").join(name);

    if !group_dir.exists() {
        eprintln!("Couldn't find config directory for gruop {group_dir:?}. Aborting.");
        bail!("Couldn't find group config directory.");
    }

    // Read the `group.yml` from the group directory.
    // Return a default config if the group config doesn't exist.
    let config = match read_yaml::<GroupConfig>(&group_dir, "group") {
        Ok(config) => config,
        Err(error) => match error {
            Error::FileNotFound(_, _) => GroupConfig::default(),
            _ => bail!(error),
        },
    };

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

        read_entry(
            &group_dir,
            Path::new(""),
            entry,
            &mut directory,
            config.target_directory.clone(),
            template_vars,
        )?;
    }

    Ok(Group {
        name: name.to_string(),
        config,
        directory,
    })
}
