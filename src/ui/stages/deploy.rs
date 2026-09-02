//! The report for the deploy phase of a run.

use std::collections::BTreeMap;

use anyhow::Result;

use super::{path_operation_row, print_header, print_path_table};
use crate::{
    changeset::{Changeset, FileOperation, PathOperation},
    error::Error,
    state::{PackageManager, ServiceManager},
    ui::{diff::Diff, style_path, theme::Stylize},
};

/// Print all changes that'll be applied to the system to reach the desired
/// state, including content diffs for modified files.
pub fn handle_deploy(changeset: &Changeset) -> Result<()> {
    print_header("Deployment");

    if !changeset.package_installs.is_empty() {
        let mut sorted: BTreeMap<PackageManager, Vec<&String>> = BTreeMap::new();
        for package in &changeset.package_installs {
            sorted
                .entry(package.manager)
                .or_default()
                .push(&package.name);
        }

        for (manager, packages) in sorted {
            println!("Packages ({manager}):");
            for package in packages {
                println!("  {} {package}", "+".addition());
            }
        }
        println!();
    }

    if !changeset.path_operations.is_empty() {
        // Print the table with all changed properties for each path.
        let rows: Vec<_> = changeset
            .path_operations
            .iter()
            .map(path_operation_row)
            .collect();
        print_path_table(format!("Paths to {}", "deploy".addition()), &rows);

        // Print the diffs of all modified files after the table.
        for op in &changeset.path_operations {
            let PathOperation::File(FileOperation::Modify {
                path,
                content: Some(desired),
                ..
            }) = op
            else {
                continue;
            };

            let actual = std::fs::read(path)
                .map_err(|err| Error::IoPath(path.clone(), "reading file for diff.", err))?;
            let diff = Diff::for_deploy(&actual, desired);

            println!(
                "\nChanges for path {}",
                style_path(path, |name| name.highlight())
            );
            println!("{}", diff.format());
        }
        println!();
    }

    if !changeset.service_enables.is_empty() {
        let mut sorted: BTreeMap<ServiceManager, Vec<&crate::changeset::ServiceEnable>> =
            BTreeMap::new();
        for service in &changeset.service_enables {
            sorted.entry(service.manager).or_default().push(service);
        }

        for (manager, services) in sorted {
            println!("Services ({manager}):");
            for service in services {
                if service.start {
                    println!("  {} {} (will be started)", "+".addition(), service.name);
                } else {
                    println!("  {} {}", "+".addition(), service.name);
                }
            }
        }
        println!();
    }

    Ok(())
}
