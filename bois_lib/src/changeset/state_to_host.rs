use std::{
    fs::read_to_string,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use users::{get_group_by_gid, get_user_by_uid};

use crate::{
    config::Configuration,
    error::Error,
    state::{State, file::Entry, group::Group, host::Host},
    system_state::SystemState,
};

use super::{
    Changeset, DirectoryOperation, FileOperation, PackageInstall, PathOperation,
    helper::equal_permissions,
};

pub fn create_changeset(
    config: &Configuration,
    state: &State,
    system_state: &mut SystemState,
) -> Result<Changeset> {
    // Create changeset for missing packages.
    let package_installs = handle_packages(state, system_state)?;

    let mut path_operations = Vec::new();
    // Create changeset for files and system services on host config.
    let host_changeset = handle_host(config, &state.host, system_state)?;
    path_operations.extend(host_changeset);

    // Create changeset for files and system services on group configs.
    for group in state.host.groups.iter() {
        let group_changset = handle_group(config, group, system_state)?;
        path_operations.extend(group_changset);
    }

    Ok(Changeset {
        package_installs,
        path_operations,
        ..Default::default()
    })
}

/// Detect any packages that're missing on the current config and queue them for installation.
fn handle_packages(state: &State, system_state: &mut SystemState) -> Result<Vec<PackageInstall>> {
    let mut installs = Vec::new();

    // Compare all desired packages in the top-level config with the currently installed one's.
    for (manager, packages) in state.packages.iter() {
        // We look at all installed packages, including dependencies.
        // In caes some desired package has already been installed as a dependency,
        // we shouldn't try to re-install it.
        let installed_packages = system_state.packages(*manager)?;
        for package in packages {
            // If a package is not found, schedule it to be installed.
            if !installed_packages.contains(package) {
                installs.push(PackageInstall {
                    manager: *manager,
                    name: package.clone(),
                })
            }
        }
    }

    Ok(installs)
}

/// Create the changeset that's needed to reach the desired state of the [HostConfig] from the
/// current system's state.
fn handle_host(
    config: &Configuration,
    host: &Host,
    _system_state: &mut SystemState,
) -> Result<Vec<PathOperation>> {
    let mut changeset = Vec::new();

    for entry in host.directory.entries.iter() {
        handle_entry(&config.target_dir, entry, &mut changeset)?;
    }

    Ok(changeset)
}

/// Create the changeset that's needed to reach the desired state of a given [GroupConfig] from the
/// current system's state.
fn handle_group(
    config: &Configuration,
    group: &Group,
    _system_state: &mut SystemState,
) -> Result<Vec<PathOperation>> {
    let mut changeset = Vec::new();

    for entry in group.directory.entries.iter() {
        handle_entry(&config.target_dir, entry, &mut changeset)?;
    }

    Ok(changeset)
}

fn handle_entry(root: &PathBuf, entry: &Entry, changeset: &mut Vec<PathOperation>) -> Result<()> {
    match entry {
        Entry::File(file) => {
            let path = file.file_path(root);

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

                changeset.push(PathOperation::File(change));

                return Ok(());
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
            if !equal_permissions(metadata.permissions().mode(), file.config.permissions()) {
                modified_permissions = Some(file.config.permissions());
            }

            // Compare owner
            let uid = metadata.uid();
            let user = get_user_by_uid(uid)
                .ok_or_else(|| anyhow!("Couldn't get username for uid {uid} on file {path:?}"))?;
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
                changeset.push(PathOperation::File(change));
            }
        }
        Entry::Directory(dir) => {
            let path = dir.file_path(root);

            // Check whether the target directory exists.
            // If it doesn't, we must push a change to create the directory.
            if !path.exists() {
                let change = DirectoryOperation::Create {
                    path,
                    permissions: dir.config.permissions(),
                    owner: dir.config.owner(),
                    group: dir.config.group(),
                };

                changeset.push(PathOperation::Directory(change));

                for entry in dir.entries.iter() {
                    handle_entry(root, entry, changeset)?;
                }
                return Ok(());
            }

            // At this point we know that the directory already exists.
            // We now have to check for any changes and whether we have to modify the directory.
            let mut modified_permissions = None;
            let mut modified_owner = None;
            let mut modified_group = None;

            let metadata = path
                .metadata()
                .map_err(|err| Error::IoPath(path.clone(), "reading metadata", err))?;

            // Check whether permissions patch
            if !equal_permissions(metadata.permissions().mode(), dir.config.permissions()) {
                modified_permissions = Some(dir.config.permissions());
            }

            // Compare owner
            let uid = metadata.uid();
            let user = get_user_by_uid(uid)
                .ok_or_else(|| anyhow!("Couldn't get username for uid {uid} on file {path:?}"))?;
            if user.name().to_string_lossy() != dir.config.owner() {
                modified_owner = Some(dir.config.owner())
            }

            // Compare group
            let gid = metadata.gid();
            let group = get_group_by_gid(gid)
                .ok_or_else(|| anyhow!("Couldn't get groupname for gid {gid}"))?;
            if group.name().to_string_lossy() != dir.config.group() {
                modified_group = Some(dir.config.group())
            }

            // If anything has been modified, push a change.
            if modified_owner.is_some()
                || modified_group.is_some()
                || modified_permissions.is_some()
            {
                let change = DirectoryOperation::Modify {
                    path,
                    permissions: modified_permissions,
                    owner: modified_owner,
                    group: modified_group,
                };
                changeset.push(PathOperation::Directory(change));
            }

            for entry in dir.entries.iter() {
                handle_entry(root, entry, changeset)?;
            }
        }
    }

    Ok(())
}
