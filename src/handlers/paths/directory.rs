use std::{
    fs::{Permissions, set_permissions},
    os::unix::fs::PermissionsExt,
    path::Path,
};

use anyhow::{Context, Result};
use file_owner::PathExt;
use log::warn;

use crate::{error::Error, ui::theme::Stylize};

pub fn create_directory(path: &Path, mode: u32, owner: &str, group: &str) -> Result<()> {
    // A previous change might have already created this directory.
    if !path.exists() {
        println!("{} directory at {path:?}", "Creating".addition());
        std::fs::create_dir(path)
            .map_err(|err| Error::IoPath(path.to_path_buf(), "creating directory.", err))?;
    }

    set_permissions(path, Permissions::from_mode(mode))?;

    path.set_owner(owner)
        .map_err(|err| Error::FileOwnership(path.to_path_buf(), "setting owner", err))?;

    path.set_group(group)
        .map_err(|err| Error::FileOwnership(path.to_path_buf(), "setting group", err))?;

    Ok(())
}

pub fn modify_directory(
    path: &Path,
    mode: &Option<u32>,
    owner: &Option<String>,
    group: &Option<String>,
) -> Result<()> {
    println!("{} directory at {path:?}", "Modifying".change());
    if let Some(mode) = mode {
        set_permissions(path, Permissions::from_mode(*mode))?;
    }

    if let Some(owner) = owner {
        path.set_owner(owner.as_str())
            .map_err(|err| Error::FileOwnership(path.to_path_buf(), "setting owner", err))?;
    }

    if let Some(group) = group {
        path.set_group(group.as_str())
            .map_err(|err| Error::FileOwnership(path.to_path_buf(), "setting group", err))?;
    }

    Ok(())
}

/// Remove a no-longer-desired managed directory: "delete if empty".
/// If the user or some program put unmanaged files into the directory,
/// it's left alone.
pub fn cleanup_directory(path: &Path) -> Result<()> {
    // The directory might already be gone. That's fine.
    if path.symlink_metadata().is_err() {
        return Ok(());
    }

    let is_empty = path
        .read_dir()
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(false);

    if !is_empty {
        warn!(
            "Not removing directory {path:?}: it still contains files \
             that aren't managed by bois."
        );
        return Ok(());
    }

    println!("{} directory at {path:?}", "Removing".removal());
    std::fs::remove_dir(path)
        .map_err(|err| Error::IoPath(path.to_path_buf(), "removing directory", err))?;

    Ok(())
}

/// Remove a directory that exists at the path of file to-be-deployed.
///
/// A non-empty directory is treated as a hard error, as we don't want to silently wipe
/// directories full of data.
pub fn remove_conflicting_directory(path: &Path) -> Result<()> {
    // The directory might already be gone. That's fine.
    if path.symlink_metadata().is_err() {
        return Ok(());
    }

    println!("{} directory at {path:?}", "Removing".removal());
    std::fs::remove_dir(path)
        .map_err(|err| Error::IoPath(path.to_path_buf(), "removing directory", err))
        .context(format!(
            "A conflicting directory exists at {path:?} \
             and could not be removed (is it non-empty?). \
             Move its contents away and re-run."
        ))?;

    Ok(())
}
