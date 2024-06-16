use std::collections::BTreeMap;

use crate::{
    changeset::{Change, Changeset, PackageOperation, PathOperation},
    constants::{CURRENT_GROUP, CURRENT_USER},
    handlers::packages::PackageManager,
};

pub fn print_package_additions(changes: &Changeset) {
    let mut sorted_changes: BTreeMap<PackageManager, Vec<String>> = BTreeMap::new();

    for change in changes.iter() {
        if let Change::PackageChange(PackageOperation::Add { manager, name }) = change {
            let list = sorted_changes.entry(*manager).or_default();
            list.push(name.clone());
        }
    }

    for (manager, packages) in sorted_changes {
        println!("{manager}");
        for package in packages {
            println!("  - {package}");
        }
    }
}

pub fn print_path_changes(changes: &Changeset) {
    for change in changes {
        let op = match change {
            Change::PackageChange(_) => continue,
            Change::PathChange(change) => change,
        };

        match op {
            PathOperation::File(op) => match op {
                crate::changeset::FileOperation::Create {
                    path,
                    permissions,
                    owner,
                    group,
                    ..
                } => {
                    println!("New file:");
                    println!("  Path: {path:?}");
                    println!("  Mod: {permissions:#o}");
                    // Don't show user/group when it's the default user/group.
                    if *owner != *CURRENT_USER {
                        println!("  Owner: {owner}");
                    }
                    if *group != *CURRENT_GROUP {
                        println!("  Group: {group}");
                    }
                }
                crate::changeset::FileOperation::Modify {
                    path,
                    permissions,
                    owner,
                    group,
                    ..
                } => {
                    println!("Changes for file:");
                    println!("  Path: {path:?}");
                    if let Some(permissions) = permissions {
                        println!("  Mod: {permissions:#o}");
                    }
                    if let Some(owner) = owner {
                        println!("  Owner: {owner}");
                    }
                    if let Some(group) = group {
                        println!("  Group: {group}");
                    }
                }
                crate::changeset::FileOperation::Delete => continue,
            },
            PathOperation::Directory(op) => match op {
                crate::changeset::DirectoryOperation::Create {
                    path,
                    permissions,
                    owner,
                    group,
                } => {
                    println!("New directory:");
                    println!("  Path: {path:?}");
                    println!("  Mod: {permissions:#o}");
                    if *owner != *CURRENT_USER {
                        println!("  Owner: {owner}");
                    }
                    if *group != *CURRENT_GROUP {
                        println!("  Group: {group}");
                    }
                }
                crate::changeset::DirectoryOperation::Modify {
                    path,
                    permissions,
                    owner,
                    group,
                } => {
                    println!("New directory:");
                    println!("  Path: {path:?}");
                    if let Some(permissions) = permissions {
                        println!("  Mod: {permissions:#o}");
                    }
                    if let Some(owner) = owner {
                        println!("  Owner: {owner}");
                    }
                    if let Some(group) = group {
                        println!("  Group: {group}");
                    }
                }
                crate::changeset::DirectoryOperation::Delete { .. } => continue,
            },
        }
    }
}
