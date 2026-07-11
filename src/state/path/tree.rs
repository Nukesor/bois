use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{config::directory::DirectoryConfig, state::path::file::File};

/// This is the full, resolved representation of one or more configuration directories.
///
/// During the aggregation step, the hosts configuration files, as well as all configuration
/// files of its enabled groups will be inserted into this tree.
///
/// The file paths of this tree are **absolute**, which means that relative paths and all
/// path overrides on group, directory and file basis are resolved.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tree {
    tree: BTreeMap<String, Node>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
    File {
        file: Box<File>,
    },
    Directory {
        directory: DirectoryConfig,
        entries: BTreeMap<String, Node>,
    },
    Symlink {
        // A relative or absolute symlink target path.
        target: PathBuf,
    },
}

impl Tree {
    /// Add a new node to the tree.
    ///
    /// As the tree only stores absolute paths, this tree
    fn add_node(_path: PathBuf) -> Result<()> {
        Ok(())
    }
}
