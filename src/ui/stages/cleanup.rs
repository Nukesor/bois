//! The report for the cleanup phase of a run.

use std::collections::BTreeMap;

use super::{PathRow, print_header, print_path_table};
use crate::{
    changeset::{Changeset, FileType, PathOperation},
    state::{PackageManager, ServiceManager},
    ui::{style_path, theme::Stylize},
};

/// Print everything from the previous run deployed that's no longer part of the
/// desired state and needs to be cleaned up.
pub fn handle_cleanup(changeset: &Changeset) {
    print_header("Cleanup");

    if !changeset.service_disables.is_empty() {
        let mut sorted: BTreeMap<ServiceManager, Vec<&String>> = BTreeMap::new();
        for service in &changeset.service_disables {
            sorted
                .entry(service.manager)
                .or_default()
                .push(&service.name);
        }

        for (manager, services) in sorted {
            println!("Services ({manager}):");
            for service in services {
                println!("  {} {} (will be stopped)", "-".removal(), service);
            }
        }
        println!();
    }

    if !changeset.path_cleanup.is_empty() {
        // Print the list with all path to remove.
        let rows: Vec<_> = changeset
            .path_cleanup
            .iter()
            .map(|op| {
                let filetype = match op {
                    PathOperation::File(_) => FileType::File,
                    PathOperation::Directory(_) => FileType::Directory,
                };
                PathRow {
                    path: format!(
                        "{} {}",
                        filetype.emoji(),
                        style_path(op.path(), |name| name.removal().bold())
                    ),
                    ..Default::default()
                }
            })
            .collect();
        print_path_table(format!("Paths to {}", "remove".removal()), &rows);
        println!();
    }

    if !changeset.package_uninstalls.is_empty() {
        let mut sorted: BTreeMap<PackageManager, Vec<&String>> = BTreeMap::new();
        for package in &changeset.package_uninstalls {
            sorted
                .entry(package.manager)
                .or_default()
                .push(&package.name);
        }

        for (manager, packages) in sorted {
            println!("Packages ({manager}):");
            for package in packages {
                println!("  {} {package}", "-".removal());
            }
        }
        println!();
    }
}
