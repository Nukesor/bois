//! This module contains logic to compare the current state with the state of the last deploy.
//! This allows us to detect any untracked changes on the system that have been done since the
//! last deploy.
//! We can then inform the user about these changes, so they aren't unintentionally overwritten.
use std::{
    fs::read_to_string,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use users::{get_group_by_gid, get_user_by_uid};

use crate::{
    config::Configuration,
    error::Error,
    state::{file::Entry, group::Group, host::Host, State},
    system_state::SystemState,
};

use super::{
    helper::{equal_permissions, remove_filetype},
    Changeset, DirectoryOperation, FileOperation, PackageUninstall, PathOperation,
};

pub fn create_changeset(
    config: &Configuration,
    system_state: &mut SystemState,
    old_state: &State,
    new_state: &State,
) -> Result<Changeset> {
    // Create changeset for packages that should be cleaned up.
    let package_uninstalls = handle_packages(system_state, old_state, new_state)?;

    let mut path_operations = Vec::new();
    // Create changeset for files and system services on host config.
    let host_changeset = handle_host(config, &old_state.host, system_state)?;
    path_operations.extend(host_changeset);

    // Create changeset for files and system services on group configs.
    for group in old_state.host.groups.iter() {
        let group_changset = handle_group(config, group, system_state)?;
        path_operations.extend(group_changset);
    }

    Ok(Changeset {
        package_uninstalls,
        path_operations,
        ..Default::default()
    })
}

/// Check for any packages that were previously on the system but have been removed.
///
/// Create a PackageChange for removal, so the user can understand the changes.
/// Make sure they haven't been removed from the new desired state before though.
/// We don't want to show removed packages that aren't desired.
fn handle_packages(
    system_state: &mut SystemState,
    old_state: &State,
    new_state: &State,
) -> Result<Vec<PackageUninstall>> {
    let mut changeset = Vec::new();

    // Compare all desired packages in the old config with the currently installed ones.
    for (manager, old_packages) in old_state.packages.iter() {
        // We compare to all packages including dependencies.
        // So in case package has been demoted to a dependency but is still installed,
        // it won't show up as a detected change.
        let installed_packages = system_state.packages(*manager)?;
        for old_package in old_packages {
            if !installed_packages.contains(old_package) {
                // Check if the removed package is supposed to be there.
                // If it isn't, just ignore this.
                if let Some(new_packages) = new_state.packages.get(manager) {
                    if !new_packages.contains(old_package) {
                        continue;
                    }
                }

                // The package has actually been removed, but is actually still desired.
                changeset.push(PackageUninstall {
                    manager: *manager,
                    name: old_package.clone(),
                })
            }
        }
    }

    Ok(changeset)
}

/// If anything happened on the deployed files of the host since the last deploy, create a
/// changeset that reflects those changes.
///
/// It's effectively the inverse logic to the `state_to_host` logic.
fn handle_host(
    config: &Configuration,
    host: &Host,
    _system_state: &mut SystemState,
) -> Result<Vec<PathOperation>> {
    let mut changeset = Vec::new();

    for entry in host.directory.entries.iter() {
        handle_entry(&config.target_dir, entry, &mut changeset)?;
    }

    // Return the reversed changeset.
    // Changes should be executed in the reverse order, as we're scanning files from the top to the
    // bottom of the file tree. But we need to remove files from the bottom to the top.
    changeset.reverse();
    Ok(changeset)
}

/// If anything happened on the deployed files of this group since the last deploy, create a
/// changeset that reflects those changes.
fn handle_group(
    config: &Configuration,
    group: &Group,
    _system_state: &mut SystemState,
) -> Result<Vec<PathOperation>> {
    let mut changeset = Vec::new();

    for entry in group.directory.entries.iter() {
        handle_entry(&config.target_dir, entry, &mut changeset)?;
    }

    // Return the reversed changeset.
    // Changes should be executed in the reverse order, as we're scanning files from the top to the
    // bottom of the file tree. But we need to remove files from the bottom to the top.
    changeset.reverse();
    Ok(changeset)
}

fn handle_entry(root: &PathBuf, entry: &Entry, changeset: &mut Vec<PathOperation>) -> Result<()> {
    match entry {
        Entry::File(file) => {
            // By default, we the destination path is the same as in the host configuration
            // directory.
            // However, if a path override exists, we always use it.
            // - If it's an absoulte path, we just use that path.
            //   This can be used to deploy files **outside** the default target dir.
            // - If it's a relative path, we just append it to the target_dir.
            let path = if let Some(path) = &file.config.path() {
                if path.is_absolute() {
                    path.clone()
                } else {
                    root.join(path)
                }
            } else {
                root.join(&file.relative_path)
            };

            // Check whether the target file exists.
            // If it doesn't, it has been deleted in the meantime.
            if !path.exists() {
                let change = FileOperation::Delete { path };
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
                modified_content = Some(content.clone());
            }

            let metadata = path
                .metadata()
                .map_err(|err| Error::IoPath(path.clone(), "reading metadata", err))?;

            // Check whether permissions patch
            let file_mode = metadata.permissions().mode();
            if !equal_permissions(file_mode, file.config.permissions()) {
                modified_permissions = Some(remove_filetype(file_mode));
            }

            // Compare owner
            let uid = metadata.uid();
            let user = get_user_by_uid(uid)
                .ok_or_else(|| anyhow!("Couldn't get username for uid {uid}"))?;
            let username = user.name().to_string_lossy();
            if username != file.config.owner() {
                modified_owner = Some(username.to_string())
            }

            // Compare group
            let gid = metadata.gid();
            let group = get_group_by_gid(gid)
                .ok_or_else(|| anyhow!("Couldn't get groupname for gid {gid}"))?;
            let group_name = group.name().to_string_lossy();
            if group_name != file.config.group() {
                modified_group = Some(group_name.to_string())
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
            // By default, we the destination path is the same as in the host configuration
            // directory.
            // However, if a path override exists, we always use it.
            // - If it's an absoulte path, we just use that path.
            //   This can be used to deploy files **outside** the default target dir.
            // - If it's a relative path, we just append it to the target_dir.
            let path = if let Some(path) = &dir.config.target_directory() {
                if path.is_absolute() {
                    path.clone()
                } else {
                    root.join(path)
                }
            } else {
                root.join(&dir.relative_path)
            };

            // Check whether the target directory exists.
            // If it doesn't, it has been deleted in the meantime.
            if !path.exists() {
                let change = DirectoryOperation::Delete { path };

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
            let file_mode = metadata.permissions().mode();
            if !equal_permissions(metadata.permissions().mode(), dir.config.permissions()) {
                modified_permissions = Some(remove_filetype(file_mode));
            }

            // Compare owner
            let uid = metadata.uid();
            let user = get_user_by_uid(uid)
                .ok_or_else(|| anyhow!("Couldn't get username for uid {uid}"))?;
            let username = user.name().to_string_lossy();
            if username != dir.config.owner() {
                modified_owner = Some(username.to_string())
            }

            // Compare group
            let gid = metadata.gid();
            let group = get_group_by_gid(gid)
                .ok_or_else(|| anyhow!("Couldn't get groupname for gid {gid}"))?;
            let group_name = group.name().to_string_lossy();
            if group_name != dir.config.group() {
                modified_group = Some(group_name.to_string())
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
