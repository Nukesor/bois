//! The recursive directory walker of the aggregation phase.
//!
//! Walks a host's or group's source directory, resolves every entry (target
//! path, metadata, templating) and inserts it into the [Tree].

use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use log::{trace, warn};
use serde_yaml::Value;

use crate::{
    config::{
        bois::Configuration,
        directory::DirectoryConfig,
        file::FileConfig,
        group::GroupConfig,
        helper::{expand_home, read_yaml},
        host::HostConfig,
    },
    constants::{CURRENT_GROUP, CURRENT_USER},
    error::Error,
    state::path::{
        DirectoryMeta,
        DirectoryPermissions,
        FileContent,
        FileState,
        Source,
        Tree,
        tree::Origin,
    },
    templating::render_template,
};

pub mod file;

use file::SourceFile;

/// The defaults for file/directory metadata of a single source (host or group).
/// Lowest layer of the `defaults < directory config < file config` cascade.
#[derive(Clone, Debug, Default)]
pub struct Defaults {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub file_mode: Option<u32>,
    pub directory_mode: Option<u32>,
}

/// Everything the walker needs to know about the source it's currently reading.
pub struct WalkContext<'a> {
    /// The absolute path of the source directory (`<bois_dir>/hosts/<name>` or
    /// `<bois_dir>/groups/<name>`).
    pub source_dir: PathBuf,
    /// The source directory relative to the bois dir, e.g. `hosts/strelok`.
    /// Used to build [Source] infos.
    pub source_prefix: PathBuf,
    /// `host:<name>` or `group:<name>`, used to build [Source] infos.
    pub origin: Origin,
    /// The metadata defaults of this source.
    pub defaults: Defaults,
    /// The source's baseline `cleanup.directories` setting. Directory
    /// `bois.yml`s can override it per subtree during the walk.
    pub cleanup_directories: bool,
    /// The absolute path to the target directory of this context's source.
    /// I.e. the global `target_dir`, or the host's/group's `target_directory` override.
    /// Relative path overrides of directories resolve against this value.
    pub target_dir: PathBuf,
    /// The templating variables of the current host.
    pub vars: &'a Value,
}

impl<'a> WalkContext<'a> {
    pub fn for_host(
        config: &Configuration,
        host_config: &HostConfig,
        name: &str,
        vars: &'a Value,
    ) -> Result<Self> {
        Ok(WalkContext {
            source_dir: config.bois_dir.join("hosts").join(name),
            source_prefix: PathBuf::from("hosts").join(name),
            origin: Origin::Host(name.into()),
            defaults: Defaults {
                owner: host_config.file_defaults.owner.clone(),
                group: host_config.file_defaults.group.clone(),
                file_mode: host_config.file_defaults.file_mode,
                directory_mode: host_config.file_defaults.directory_mode,
            },
            cleanup_directories: host_config.cleanup.directories.unwrap_or(false),
            target_dir: resolve_target_dir(&config.target_dir, &host_config.target_directory)?,
            vars,
        })
    }

    pub fn for_group(
        config: &Configuration,
        group_config: &GroupConfig,
        name: &str,
        vars: &'a Value,
    ) -> Result<Self> {
        Ok(WalkContext {
            source_dir: config.bois_dir.join("groups").join(name),
            source_prefix: PathBuf::from("groups").join(name),
            origin: Origin::Group(name.into()),
            defaults: Defaults {
                owner: group_config.defaults.owner.clone(),
                group: group_config.defaults.group.clone(),
                file_mode: group_config.defaults.file_mode,
                directory_mode: group_config.defaults.directory_mode,
            },
            cleanup_directories: group_config.cleanup.directories.unwrap_or(false),
            target_dir: resolve_target_dir(&config.target_dir, &group_config.target_directory)?,
            vars,
        })
    }

    /// The [Source] for an entry at the given path (relative to the source dir).
    fn source(&self, relative_path: &Path) -> Source {
        Source::new(self.origin.clone(), self.source_prefix.join(relative_path))
    }
}

/// Determine a source's effective target directory: the global target dir, or
/// the source's `target_directory` override, which must be absolute.
fn resolve_target_dir(target_dir: &Path, over_ride: &Option<PathBuf>) -> Result<PathBuf> {
    match over_ride {
        Some(path) => {
            let path = expand_home(path);
            if path.is_absolute() {
                Ok(path)
            } else {
                bail!("`target_directory` {path:?} must be an absolute path");
            }
        }
        None => Ok(target_dir.to_path_buf()),
    }
}

/// The configuration file names that are found in the root of host and gropu
/// directories. Those should never be deployed.
const ROOT_MARKER_FILES: [&str; 6] = [
    "host.yml",
    "host.yaml",
    "group.yml",
    "group.yaml",
    "vars.yml",
    "vars.yaml",
];

/// Walk the top level of a host/group source directory and insert all
/// deployable entries into the tree.
///
/// The root of the source directory itself maps to the source's target directory, which is never
/// managed by bois, so no directory node is created for it. (E.g. `~/.config` for user configs).
pub fn walk_source(ctx: &WalkContext, tree: &mut Tree) -> Result<()> {
    for entry in read_dir_sorted(&ctx.source_dir)? {
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip all bois-internal configuration files at the source root.
        if ROOT_MARKER_FILES.contains(&file_name.as_str()) {
            continue;
        }

        let relative_path = PathBuf::from(&file_name);
        let default_target = ctx.target_dir.join(&file_name);
        walk_entry(
            ctx,
            &entry,
            &relative_path,
            default_target,
            ctx.cleanup_directories,
            tree,
        )?;
    }

    Ok(())
}

/// Dispatch a directory entry to the file/directory handler.
///
/// `cleanup_directories` is the setting recursively inherited from the
/// parent-directory/host/group config.
fn walk_entry(
    ctx: &WalkContext,
    entry: &DirEntry,
    relative_path: &Path,
    default_target: PathBuf,
    cleanup_directories: bool,
    tree: &mut Tree,
) -> Result<()> {
    let path = entry.path();
    if path.is_dir() {
        walk_directory(
            ctx,
            relative_path,
            default_target,
            cleanup_directories,
            tree,
        )
    } else if path.is_file() {
        handle_file(ctx, relative_path, &default_target, tree)
    } else {
        warn!("Ignoring unsupported filesystem entry (symlink/socket/...) at {path:?}");
        Ok(())
    }
}

/// The configuration file names that are found in source sub-directories.
const CONFIG_MARKER_FILES: [&str; 2] = ["bois.yml", "bois.yaml"];

/// Recursively read a directory inside a source directory and insert it and
/// all its contents into the tree.
///
/// `default_target` is basically `{{parent_target}}/{{directory_name}}`, in the case that the
/// directory does not provide its own path override.
///
/// A directory whose target resolves to an [unmanaged path](is_unmanaged_target)
/// gets no tree node: its children deploy normally, but the directory itself is
/// never created, modified or removed by bois.
fn walk_directory(
    ctx: &WalkContext,
    relative_path: &Path,
    default_target: PathBuf,
    cleanup_directories: bool,
    tree: &mut Tree,
) -> Result<()> {
    let directory_path = ctx.source_dir.join(relative_path);
    trace!("Entered directory {directory_path:?}");

    // Read the `bois.yml` from the directory if it exists.
    let has_config =
        directory_path.join("bois.yml").exists() || directory_path.join("bois.yaml").exists();
    let config = if has_config {
        read_yaml::<DirectoryConfig>(&directory_path, "bois")?
    } else {
        DirectoryConfig::default()
    };

    // The cleanup setting cascades: this directory's bois.yml overrides the
    // value inherited from its parent (ultimately the host/group config), for
    // itself and everything below it.
    let cleanup_directories = config.cleanup.directories.unwrap_or(cleanup_directories);

    // An explicit path override replaces the default target for this directory
    // and thereby for all of its children.
    let target = match config.path() {
        Some(path) => {
            if path.is_absolute() {
                path
            } else {
                ctx.target_dir.join(path)
            }
        }
        None => default_target,
    };

    if is_unmanaged_target(ctx, &target) {
        // We ignore any permission settings on directories that are either `/` or the default path
        // of a group/host. Root shouldn't be managed and the default path basically acts as
        // a sort of "mount point" and should mostly never be managed by us anyway (i.e.
        // `~/.config` or `/etc`)
        if config.mode.is_some() || config.owner.is_some() || config.group.is_some() {
            warn!(
                "Ignoring mode/owner/group of {directory_path:?}: \
                 Folder {target:?} is not managed by bois."
            );
        }
    } else {
        let meta = DirectoryMeta {
            permissions: resolve_directory_permissions(ctx, &config),
            cleanup: cleanup_directories,
        };

        tree.insert_directory(&target, meta, ctx.source(relative_path))?;
    }

    // Now recurse into the directory's entries.
    for entry in read_dir_sorted(&directory_path)? {
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip the directory's own configuration file.
        if CONFIG_MARKER_FILES.contains(&file_name.as_str()) {
            continue;
        }

        let relative_path = relative_path.join(&file_name);
        let default_target = target.join(&file_name);
        walk_entry(
            ctx,
            &entry,
            &relative_path,
            default_target,
            cleanup_directories,
            tree,
        )?;
    }

    Ok(())
}

/// Build a source directory's permissions from the inherited parent context and
/// any overrides from the directory config.
///
/// The current directory configs always take precedence over inherited context.
/// If no instructions exist at all, we simply return [DirectoryPermissions::Default].
fn resolve_directory_permissions(
    ctx: &WalkContext,
    config: &DirectoryConfig,
) -> DirectoryPermissions {
    let mode = config.mode.or(ctx.defaults.directory_mode);
    let owner = config.owner.clone().or_else(|| ctx.defaults.owner.clone());
    let group = config.group.clone().or_else(|| ctx.defaults.group.clone());

    if mode.is_some() || owner.is_some() || group.is_some() {
        DirectoryPermissions::Declared { mode, owner, group }
    } else {
        DirectoryPermissions::Default
    }
}

/// Paths that bois never manages: the filesystem root and the host's/group's target
/// directory itself.
fn is_unmanaged_target(ctx: &WalkContext, target: &Path) -> bool {
    target == Path::new("/") || target == ctx.target_dir
}

/// Read and insert a single source file.
///
/// `default_target_path` is where the file would end up if it does not have its own path override.
fn handle_file(
    ctx: &WalkContext,
    relative_path: &Path,
    default_target_path: &Path,
    tree: &mut Tree,
) -> Result<()> {
    let path = ctx.source_dir.join(relative_path);
    trace!("Reading file {path:?}");
    let SourceFile {
        mode: source_mode,
        config,
        mut content,
    } = SourceFile::from_path(&path)?;

    // Perform templating, if enabled.
    if config.template {
        match content {
            FileContent::Text(ref text) => {
                let rendered = render_template(text, ctx.vars, &config.delimiters)
                    .context(format!("Error for template at {path:?}"))?;
                content = FileContent::Text(rendered);
            }
            FileContent::Binary(_) => {
                warn!("Ignoring 'template: true' on binary file {path:?}");
            }
        }
    }

    let target = resolve_file_target(ctx, &config, default_target_path);

    let file_state = FileState {
        content,
        // The file-type bits of the source file's on-disk mode are masked, we
        // only care about the permission (+ setuid/setgid/sticky) bits.
        mode: config
            .mode
            .or(ctx.defaults.file_mode)
            .unwrap_or(source_mode & 0o7777),
        owner: config
            .owner
            .clone()
            .or_else(|| ctx.defaults.owner.clone())
            .unwrap_or_else(|| CURRENT_USER.clone()),
        group: config
            .group
            .clone()
            .or_else(|| ctx.defaults.group.clone())
            .unwrap_or_else(|| CURRENT_GROUP.clone()),
        source: ctx.source(relative_path),
    };

    tree.insert_file(&target, file_state)?;

    Ok(())
}

/// Resolve the final absolute target path of a file.
///
/// - A `path` override in the file's config wins. Relative overrides resolve against the source's
///   target directory, absolute ones are used as is. A trailing `/` means "the file goes *into*
///   this directory"; without one, the override is the full destination path including the file
///   name.
/// - Otherwise the file lands in its parent's target directory under its own name.
/// - A `rename` override replaces the resulting file name.
fn resolve_file_target(
    ctx: &WalkContext,
    config: &FileConfig,
    default_target_path: &Path,
) -> PathBuf {
    let mut target = match config.path() {
        Some(path) => {
            let is_dir_style = path.to_string_lossy().ends_with('/');
            let path = if path.is_absolute() {
                path
            } else {
                ctx.target_dir.join(path)
            };

            if is_dir_style {
                // `path: /usr/local/bin/` deploys the file into that directory.
                let file_name = default_target_path
                    .file_name()
                    .expect("target paths always have a file name");
                path.join(file_name)
            } else {
                path
            }
        }
        None => default_target_path.to_path_buf(),
    };

    // If a rename is requested, replace the file name.
    if let Some(file_name) = &config.rename {
        target.set_file_name(file_name);
    }

    target
}

/// Read a directory's entries sorted by file name, so that the aggregation order (and
/// thereby warning/conflict order) is deterministic and idempotent.
fn read_dir_sorted(path: &Path) -> Result<Vec<DirEntry>> {
    let entries = std::fs::read_dir(path)
        .map_err(|err| Error::IoPath(path.to_path_buf(), "reading directory", err))?;

    let mut entries: Vec<DirEntry> = entries
        .collect::<Result<_, _>>()
        .map_err(|err| Error::IoPath(path.to_path_buf(), "reading directory entry", err))?;

    entries.sort_by_key(|entry| entry.file_name());

    Ok(entries)
}
