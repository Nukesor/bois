use std::{fs::File, io::Write, os::unix::fs::PermissionsExt, path::Path};

use anyhow::Result;
use file_owner::PathExt;

use crate::error::Error;

pub fn create_file(
    path: &Path,
    content: &[u8],
    permissions: &u32,
    owner: &str,
    group: &str,
) -> Result<()> {
    let mut file = File::create(path)
        .map_err(|err| Error::IoPath(path.to_path_buf(), "creating file.", err))?;

    file.write_all(content)
        .map_err(|err| Error::IoPath(path.to_path_buf(), "writing to file.", err))?;

    let metadata = file.metadata().map_err(|err| {
        Error::IoPath(path.to_path_buf(), "reading metadata of created file.", err)
    })?;

    metadata.permissions().set_mode(*permissions);

    path.set_owner(owner)
        .map_err(|err| Error::FileOwnership(path.to_path_buf(), "setting owner", err))?;

    path.set_group(group)
        .map_err(|err| Error::FileOwnership(path.to_path_buf(), "setting group", err))?;

    Ok(())
}

pub fn modify_file(
    path: &Path,
    content: &Option<Vec<u8>>,
    permissions: &Option<u32>,
    owner: &Option<String>,
    group: &Option<String>,
) -> Result<()> {
    // Get options to read/write the file.
    let mut file_options = File::options();
    file_options.read(true).write(true);

    // If we plan to overwrite the file's contents, also truncate it.
    if content.is_some() {
        file_options.truncate(true);
    }

    // Now we open the file.
    let mut file = file_options
        .open(path)
        .map_err(|err| Error::IoPath(path.to_path_buf(), "opening file.", err))?;

    // Immediately write all contents to the file.
    if let Some(content) = content {
        file.write_all(content)
            .map_err(|err| Error::IoPath(path.to_path_buf(), "writing to file.", err))?;
    }

    if let Some(permissions) = permissions {
        let metadata = file.metadata().map_err(|err| {
            Error::IoPath(path.to_path_buf(), "reading metadata of created file.", err)
        })?;
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

pub fn remove_file(path: &Path) -> Result<()> {
    // This shouldn't happen, but let's handle it anyway.
    if !path.exists() {
        return Ok(());
    }

    std::fs::remove_file(path)
        .map_err(|err| Error::IoPath(path.to_path_buf(), "removing file", err))?;

    Ok(())
}
