use std::{os::unix::fs::PermissionsExt, path::Path};

use anyhow::Result;
use file_owner::PathExt;

use crate::error::Error;

pub fn create_directory(path: &Path, permissions: &u32, owner: &str, group: &str) -> Result<()> {
    // A previous change might have already created this directory.
    if !path.exists() {
        // Create the directory
        std::fs::create_dir(path)
            .map_err(|err| Error::IoPath(path.to_path_buf(), "creating directory.", err))?;
    }

    let metadata = std::fs::metadata(path)
        .map_err(|err| Error::IoPath(path.to_path_buf(), "reading metadata", err))?;
    metadata.permissions().set_mode(*permissions);

    path.set_owner(owner)
        .map_err(|err| Error::FileOwnership(path.to_path_buf(), "setting owner", err))?;

    path.set_group(group)
        .map_err(|err| Error::FileOwnership(path.to_path_buf(), "setting group", err))?;

    Ok(())
}

pub fn modify_directory(
    path: &Path,
    permissions: &Option<u32>,
    owner: &Option<String>,
    group: &Option<String>,
) -> Result<()> {
    if let Some(permissions) = permissions {
        let metadata = std::fs::metadata(path)
            .map_err(|err| Error::IoPath(path.to_path_buf(), "reading metadata", err))?;

        metadata.permissions().set_mode(*permissions);
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

pub fn remove_directory(path: &Path) -> Result<()> {
    // This shouldn't happen, but let's handle it anyway.
    if !path.exists() {
        return Ok(());
    }

    std::fs::remove_dir(path)
        .map_err(|err| Error::IoPath(path.to_path_buf(), "removing directory", err))?;

    Ok(())
}
