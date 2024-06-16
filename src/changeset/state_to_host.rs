use std::{
    fs::read_to_string,
    os::unix::fs::{MetadataExt, PermissionsExt},
};

use anyhow::{anyhow, Result};
use users::{get_group_by_gid, get_user_by_uid};

use crate::{
    config::Configuration,
    error::Error,
    state::{group::Group, host::Host, State},
    system_state::SystemState,
};

use super::{Change, ChangeSet, FileOperation, PackageOperation, PathOperation};

pub fn create_changeset(
    config: &Configuration,
    state: &State,
    system_state: &mut SystemState,
) -> Result<Option<ChangeSet>> {
    let mut changeset = Vec::new();

    // Create changeset for missing packages.
    let package_changeset = handle_packages(state, system_state)?;
    changeset.extend(package_changeset);

    // Create changeset for files and system services on host config.
    let host_changeset = handle_host(config, &state.host, system_state)?;
    changeset.extend(host_changeset);

    // Create changeset for files and system services on group configs.
    for group in state.host.groups.iter() {
        let group_changset = handle_group(group, system_state)?;
        changeset.extend(group_changset);
    }

    if changeset.is_empty() {
        Ok(None)
    } else {
        Ok(Some(changeset))
    }
}

/// Detect any packages that're missing on the current config and queue them for installation.
fn handle_packages(state: &State, system_state: &mut SystemState) -> Result<ChangeSet> {
    let mut changeset = Vec::new();

    // Compare all desired packages in the top-level config with the currently installed one's.
    for (manager, packages) in state.packages.iter() {
        // We look at all installed packages, including dependencies.
        // In caes some desired package has already been installed as a dependency,
        // we shouldn't try to re-install it.
        let installed_packages = system_state.packages(*manager)?;
        for package in packages {
            // If a package is not found, schedule it to be installed.
            if !installed_packages.contains(package) {
                changeset.push(Change::PackageChange(PackageOperation::Add {
                    manager: *manager,
                    name: package.clone(),
                }))
            }
        }
    }

    Ok(changeset)
}

/// Create the changeset that's needed to reach the desired state of the [HostConfig] from the
/// current system's state.
fn handle_host(
    config: &Configuration,
    host: &Host,
    _system_state: &mut SystemState,
) -> Result<ChangeSet> {
    let mut changeset = Vec::new();

    for entry in host.directory.entries.iter() {
        match entry {
            crate::state::file::Entry::File(file) => {
                // If there's an explicit path, pick it, otherwise fall back to the relative path.
                let path = if let Some(path) = &file.config.path {
                    path.clone()
                } else {
                    config.target_dir().join(&file.relative_path)
                };

                // Check whether the target file exists.
                // If it doesn't, we must push a change to create the file.
                if !path.exists() {
                    let change = FileOperation::Create {
                        path,
                        content: file.content.clone().into_bytes(),
                        permissions: file.config.permissions(),
                        owner: file.config.owner(),
                        group: file.config.group(),
                    };

                    changeset.push(Change::PathChange(PathOperation::File(change)));
                    continue;
                }

                // At this point we know that the file already exists.
                // We now have to check for any changes and whether we have to modify the file.
                let mut modified_content = None;
                let mut modified_permissions = None;
                let mut modified_owner = None;
                let mut modified_group = None;

                // Check whether content matches
                let content = read_to_string(&path)
                    .map_err(|err| Error::IoPath(path.clone(), "reading file", err))?;
                if content.trim() != file.content.trim() {
                    modified_content = Some(file.content.clone());
                }

                let metadata = path
                    .metadata()
                    .map_err(|err| Error::IoPath(path.clone(), "reading metadata", err))?;

                // Check whether permissions patch
                if metadata.permissions().mode() != file.config.permissions() {
                    modified_permissions = Some(file.config.permissions());
                }

                // Compare owner
                let uid = metadata.uid();
                let user = get_user_by_uid(uid)
                    .ok_or_else(|| anyhow!("Couldn't get username for uid {uid}"))?;
                if user.name().to_string_lossy() != file.config.owner() {
                    modified_owner = Some(file.config.owner())
                }

                // Compare group
                let gid = metadata.gid();
                let group = get_group_by_gid(gid)
                    .ok_or_else(|| anyhow!("Couldn't get groupname for gid {gid}"))?;
                if group.name().to_string_lossy() != file.config.group() {
                    modified_group = Some(file.config.group())
                }

                // If anything has been modified, push a change.
                if modified_content.is_some()
                    || modified_owner.is_some()
                    || modified_group.is_some()
                    || modified_permissions.is_some()
                {
                    let change = FileOperation::Modify {
                        path,
                        content: modified_content.map(|str| str.into_bytes()),
                        permissions: modified_permissions,
                        owner: modified_owner,
                        group: modified_group,
                    };
                    changeset.push(Change::PathChange(PathOperation::File(change)));
                }
            }
            crate::state::file::Entry::Directory(dir) => todo!(),
        }
    }

    Ok(changeset)
}

/// Create the changeset that's needed to reach the desired state of a given [GroupConfig] from the
/// current system's state.
fn handle_group(_group: &Group, _system_state: &mut SystemState) -> Result<ChangeSet> {
    let changeset = Vec::new();

    Ok(changeset)
}
