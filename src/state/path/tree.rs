use std::{
    collections::BTreeMap,
    fmt::Display,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{error::Error, state::path::file::FileState};

/// This is the fully resolved representation of all configuration files for a given host.
///
/// This includes the source directories of the `host` and all its enabled traits.
///
/// The paths of this tree are **absolute**, which means that relative paths and all
/// path overrides on trait, directory and files have already been resolved.
///
/// The tree thereby directly mirrors the state to-be deployed on the filesystem.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    /// The root of the filesystem, which effectively represents `/`.
    pub root: BTreeMap<String, Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Node {
    File(FileState),
    Directory(DirectoryState),
}

/// A directory in the tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryState {
    /// Whether a source directory "backs" this directory, and if so with what settings.
    pub backing: DirectoryBacking,
    // TODO(backwards compatibility): alias
    #[serde(alias = "entries")]
    pub children: BTreeMap<String, Node>,
    /// Where this directory came from. For implicit directories this points at
    /// the source of the node whose target path caused the directory to exist.
    pub source: Source,
}

impl DirectoryState {
    /// Whether this directory only exists as a parent component of some other
    /// node's target path, without a source directory backing it.
    pub fn is_implicit(&self) -> bool {
        matches!(self.backing, DirectoryBacking::Implicit)
    }

    /// The settings of the backing source directory, if there is one.
    pub fn meta(&self) -> Option<&DirectoryMeta> {
        match &self.backing {
            DirectoryBacking::Backed(meta) => Some(meta),
            DirectoryBacking::Implicit => None,
        }
    }
}

/// Whether a directory in the tree is backed by a source directory.
///
/// "backed" in this this context means that there's a real directory
/// in the source directory that introduced this node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectoryBacking {
    /// A real directory in a host/trait source tree, with its resolved
    /// permission management and cleanup settings.
    Backed(DirectoryMeta),
    /// Only exists as a parent component of some other node's target path
    /// For example, `/etc` would be implicit for `/etc/udev/rules.d/`.
    Implicit,
}

/// The fully resolved settings of a source directory.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryMeta {
    /// The directory's permissions.
    pub permissions: DirectoryPermissions,
    /// Whether to remove this directory once all mentions from the config sources
    /// are removed.
    pub cleanup: bool,
}

/// The permissions a deployed source directory's should be set to.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectoryPermissions {
    /// At least one field was declared, in the directory's `dir.yml` or a
    /// `defaults` cascade.
    ///
    /// All `None` fields default to `0x755` and the current user.
    Declared {
        mode: Option<u32>,
        owner: Option<String>,
        group: Option<String>,
    },
    /// No explicit permissions have been declared.
    /// This will result in default settings (i.e. 0o755 with current user/group)
    #[default]
    Default,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    Host(String),
    Trait(String),
}

impl Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::Host(name) => write!(f, "Host '{name}'"),
            Origin::Trait(name) => write!(f, "Trait '{name}'"),
        }
    }
}

/// Describes where a node in the [Tree] originated, so conflicts and diffs
/// can point the user to the source files that caused issues.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// The host or trait.
    pub origin: Origin,
    /// The path of the source file/directory
    ///
    /// **Note:**
    /// Since this points to the source files, this path is relative to the bois directory.
    pub path: PathBuf,
}

impl Source {
    pub fn new<P: Into<PathBuf>>(origin: Origin, path: P) -> Self {
        Source {
            origin,
            path: path.into(),
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.origin, self.path.to_string_lossy())
    }
}

impl Node {
    /// The source of this node.
    pub fn source(&self) -> &Source {
        match self {
            Node::File(file) => &file.source,
            Node::Directory(dir) => &dir.source,
        }
    }

    /// Describe this node for conflict messages.
    /// Implicit directories don't exist in any source directory, so they
    /// name the node whose target path caused them instead.
    fn describe(&self) -> String {
        match self {
            Node::File(file) => format!("file from {}", file.source),
            Node::Directory(dir) if dir.is_implicit() => format!(
                "directory implicitly created for the target path of {}",
                dir.source
            ),
            Node::Directory(dir) => format!("directory from {}", dir.source),
        }
    }
}

impl Tree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the node at the given absolute path.
    pub fn get(&self, path: &Path) -> Option<&Node> {
        let (parent_components, name) = Self::split_absolute_path(path, None).ok()?;

        let mut children = &self.root;
        for component in parent_components {
            match children.get(&component) {
                Some(Node::Directory(dir)) => children = &dir.children,
                _ => return None,
            }
        }

        children.get(&name)
    }

    /// Remove the node at the given absolute path from the tree.
    ///
    /// Returns the removed node, if there was one.
    /// Parent directories are left in place, even if they become empty.
    pub fn remove(&mut self, path: &Path) -> Option<Node> {
        let (parent_components, name) = Self::split_absolute_path(path, None).ok()?;

        let mut children = &mut self.root;
        for component in parent_components {
            match children.get_mut(&component) {
                Some(Node::Directory(dir)) => children = &mut dir.children,
                _ => return None,
            }
        }

        children.remove(&name)
    }

    /// Insert a fully resolved file at an absolute target path.
    ///
    /// Missing parent directories are created as [DirectoryBacking::Implicit],
    /// with the file's source attached to them.
    ///
    /// ## Errors
    ///
    /// Errors if the target path is already occupied by a conflicting node.
    pub fn insert_file(&mut self, target: &Path, file: FileState) -> Result<(), Error> {
        let (parent_components, file_name) = Self::split_absolute_path(target, Some(&file.source))?;

        let children = self.ensure_directory_chain(target, &parent_components, &file.source)?;

        match children.get(&file_name) {
            None => {
                children.insert(file_name, Node::File(file));
                Ok(())
            }
            Some(existing) => Err(Self::conflict(
                target,
                existing,
                format!("file from {}", file.source),
            )),
        }
    }

    /// Insert a fully resolved directory at an absolute target path.
    ///
    /// If a directory already exists at the path, the two are merged accordingly
    /// if possible. See [Self::merge_directory] for info on the merge logic.
    ///
    /// ## Errors
    ///
    /// - If two directories cannot be merged
    /// - If a file is at the target path.
    pub fn insert_directory(
        &mut self,
        target: &Path,
        meta: DirectoryMeta,
        source: Source,
    ) -> Result<(), Error> {
        let (parent_components, dir_name) = Self::split_absolute_path(target, Some(&source))?;

        // Make sure the path exists or create intermediate directories in the tree.
        let parent_handle = self.ensure_directory_chain(target, &parent_components, &source)?;

        // Check if we already have a file/dir with that name.
        match parent_handle.get_mut(&dir_name) {
            // If not, create a new directory with the current config.
            None => {
                parent_handle.insert(
                    dir_name,
                    Node::Directory(DirectoryState {
                        backing: DirectoryBacking::Backed(meta),
                        children: BTreeMap::new(),
                        source,
                    }),
                );
                Ok(())
            }
            // If there is, attempt to merge the configs.
            // This will error when both directories were explicit and both had conflicting
            // settings.
            Some(Node::Directory(existing)) => {
                Self::merge_directory(target, existing, meta, source)
            }
            Some(existing @ Node::File(_)) => Err(Self::conflict(
                target,
                existing,
                format!("directory from {source}"),
            )),
        }
    }

    /// Merge a directory into an existing directory node.
    ///
    /// The rules are as follows:
    ///
    /// Implicit directories are the weakest type of directory and are always overwritten.
    /// For two explicit directories, a merge is attempted:
    /// - [`DirectoryPermissions::Default`] and `Declared`: The declared case always has precedence
    ///   and will be set without error.
    /// - Two [DirectoryPermissions::Declared]: In this case, a merge on a per-field basis is
    ///   attempted. If any conflicting declarations exist, an error is thrown and the merge fails.
    /// - `cleanup`: Can always be merged, but applies a boolean `&&`. I.e. `cleanup` is only true
    ///   when **all** sources declare that it should be cleaned up.
    ///
    /// ## Error
    ///
    /// A file at the target path is always a hard error.
    fn merge_directory(
        target: &Path,
        existing: &mut DirectoryState,
        new: DirectoryMeta,
        new_source: Source,
    ) -> Result<(), Error> {
        match &mut existing.backing {
            // The existing node was only an implicit parent so far. The real
            // source directory takes over the node's identity, keeping the
            // children.
            DirectoryBacking::Implicit => {
                existing.backing = DirectoryBacking::Backed(new);
                existing.source = new_source;
            }
            DirectoryBacking::Backed(existing_meta) => {
                existing_meta.permissions = Self::merge_permissions(
                    target,
                    existing_meta.permissions.clone(),
                    new.permissions,
                    &existing.source,
                    &new_source,
                )?;
                // Cleanup requires all backing sources to opt in.
                existing_meta.cleanup &= new.cleanup;
            }
        }

        Ok(())
    }

    /// Merge the permission management of two source directories.
    ///
    /// Any `Declared` permissions overrule `Default` permissions.
    /// In the case of two `Declared` permissions, a [Self::merge_field] is attempted on all fields.
    fn merge_permissions(
        target: &Path,
        existing: DirectoryPermissions,
        new: DirectoryPermissions,
        existing_source: &Source,
        new_source: &Source,
    ) -> Result<DirectoryPermissions, Error> {
        use DirectoryPermissions::*;

        match (existing, new) {
            (Default, Default) => Ok(Default),

            // A declaration beats default tracking.
            (declared @ Declared { .. }, Default) | (Default, declared @ Declared { .. }) => {
                Ok(declared)
            }

            // Two declarations merge per field.
            (
                Declared { mode, owner, group },
                Declared {
                    mode: new_mode,
                    owner: new_owner,
                    group: new_group,
                },
            ) => Ok(Declared {
                mode: Self::merge_field(
                    target,
                    "mode",
                    mode,
                    new_mode,
                    existing_source,
                    new_source,
                )?,
                owner: Self::merge_field(
                    target,
                    "owner",
                    owner,
                    new_owner,
                    existing_source,
                    new_source,
                )?,
                group: Self::merge_field(
                    target,
                    "group",
                    group,
                    new_group,
                    existing_source,
                    new_source,
                )?,
            }),
        }
    }

    /// Merge a single declared permission field of two directories.
    fn merge_field<T: PartialEq>(
        target: &Path,
        field_name: &str,
        existing_value: Option<T>,
        new_value: Option<T>,
        existing_source: &Source,
        new_source: &Source,
    ) -> Result<Option<T>, Error> {
        match (existing_value, new_value) {
            (existing, None) => Ok(existing),
            (None, new) => Ok(new),
            (Some(existing), Some(new)) => {
                if existing == new {
                    Ok(Some(existing))
                } else {
                    Err(Error::PathConflict(format!(
                        "Directory {target:?} has conflicting '{field_name}' declarations:\n  \
             - {existing_source}\n  - {new_source}\n\
             Align the two declarations or remove one of them."
                    )))
                }
            }
        }
    }

    fn conflict(target: &Path, existing: &Node, new: String) -> Error {
        Error::PathConflict(format!(
            "Duplicate declarations for target path {target:?}:\n  - {existing}\n  - {new}",
            existing = existing.describe()
        ))
    }

    /// Flatten the tree into a list of `(absolute path, node)` tuples.
    ///
    /// The list is ordered the following way:
    /// - parents directories before children
    /// - Alphabetically descending within each directory
    ///
    /// This effectively is the order in which we deploy.
    /// Reverse it for removals.
    pub fn flatten(&self) -> Vec<(PathBuf, &Node)> {
        let mut result = Vec::new();
        Self::flatten_level(&PathBuf::from("/"), &self.root, &mut result);
        result
    }

    // Recursively walk through a node and flatten it into a single array as described in
    // [Tree::flatten].
    fn flatten_level<'a>(
        prefix: &Path,
        children: &'a BTreeMap<String, Node>,
        result: &mut Vec<(PathBuf, &'a Node)>,
    ) {
        for (name, node) in children {
            let path = prefix.join(name);
            result.push((path.clone(), node));
            if let Node::Directory(dir) = node {
                Self::flatten_level(&path, &dir.children, result);
            }
        }
    }

    /// Walk down the tree along the given parent components, creating missing directories on the
    /// way.
    ///
    /// Returns the child map of the last parent directory.
    ///
    /// ## Errors
    ///
    /// Errors if any component on the way is a file.
    fn ensure_directory_chain(
        &mut self,
        target: &Path,
        components: &[String],
        source: &Source,
    ) -> Result<&mut BTreeMap<String, Node>, Error> {
        // Start at the root of the
        let mut children = &mut self.root;
        let mut current_path = PathBuf::from("/");

        for component in components {
            current_path.push(component);

            let node = children.entry(component.clone()).or_insert_with(|| {
                Node::Directory(DirectoryState {
                    backing: DirectoryBacking::Implicit,
                    children: BTreeMap::new(),
                    source: source.clone(),
                })
            });

            match node {
                Node::Directory(dir) => children = &mut dir.children,
                Node::File(file) => {
                    return Err(Error::PathConflict(format!(
                        "Path component {current_path:?} of target {target:?} is already a file.\n  \
                     - existing file: {}\n  - conflicting declaration: {source}",
                        file.source
                    )));
                }
            }
        }

        Ok(children)
    }

    /// Split an absolute path into its parent components and the final component.
    ///
    /// This is used in the Tree functions to have a nice data format for checking
    /// and creating parent directories.
    ///
    /// ## Caller Expectations
    ///
    /// - `path` must be absolute.
    /// - `path` must not be `/`. [Tree] does not manage the filesystem root.
    ///
    /// ## Error
    ///
    /// - Rejects any paths with relative `..` components
    fn split_absolute_path(
        path: &Path,
        source: Option<&Source>,
    ) -> Result<(Vec<String>, String), Error> {
        let source_info = source
            .map(|source| format!(" (from {source})"))
            .unwrap_or_default();

        if !path.is_absolute() {
            return Err(Error::Generic(format!(
                "Expected absolute target path, got {path:?}{source_info}. \
             This is a bug in the aggregation logic."
            )));
        }

        let mut components: Vec<String> = Vec::new();
        for component in path.components() {
            match component {
                Component::Normal(name) => components.push(name.to_string_lossy().to_string()),
                Component::ParentDir => {
                    return Err(Error::Generic(format!(
                        "Target path {path:?}{source_info} contains a '..' component. \
                     Use a path without '..' in the respective path override."
                    )));
                }
                // The leading `/` and no-op `.` components.
                _ => {}
            }
        }

        // The aggregator handles unmanaged targets like `/` itself, so this should never be
        // reached.
        let Some(name) = components.pop() else {
            return Err(Error::Generic(format!(
                "Cannot insert the filesystem root into the path tree {source_info}. \
             This is a bug in the aggregation logic."
            )));
        };

        Ok((components, name))
    }
}
