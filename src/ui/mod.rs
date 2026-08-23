//! All user-facing output of changesets and drift reports:
//! change summaries, metadata tables and content diffs.

use std::{collections::BTreeMap, fs::File, io::Write, path::Path, process::Command};

use anyhow::Result;
use comfy_table::{Attribute, Cell, CellAlignment, Column, ContentArrangement, Table, presets};
use crossterm::style::Stylize;

use crate::{
    changeset::{
        DirectoryOperation,
        Drift,
        FileOperation,
        PackageInstall,
        PackageUninstall,
        PathChange,
        PathChangeKind,
        PathOperation,
        ServiceDisable,
        ServiceEnable,
    },
    config::bois::Configuration,
    constants::{CURRENT_GROUP, CURRENT_USER},
    error::Error,
    state::{PackageManager, ServiceManager, path::FileContent},
};

pub fn print_package_uninstalls(packages: &[PackageUninstall]) {
    let mut sorted_changes: BTreeMap<PackageManager, Vec<String>> = BTreeMap::new();
    print_header("Package removals");

    for pkg in packages.iter() {
        let list = sorted_changes.entry(pkg.manager).or_default();
        list.push(pkg.name.clone());
    }

    for (manager, packages) in sorted_changes {
        println!("{}:", manager.to_string().bold());
        for package in packages {
            println!("  {} {package}", "-".red());
        }
    }
}

pub fn print_package_installs(packages: &[PackageInstall]) {
    let mut sorted_changes: BTreeMap<PackageManager, Vec<String>> = BTreeMap::new();
    print_header("Package additions");

    for pkg in packages.iter() {
        let list = sorted_changes.entry(pkg.manager).or_default();
        list.push(pkg.name.clone());
    }

    for (manager, packages) in sorted_changes {
        println!("{}:", manager.to_string().bold());
        for package in packages {
            println!("  {} {package}", "+".green());
        }
    }
}

pub fn print_service_enables(services: &[ServiceEnable]) {
    let mut sorted_changes: BTreeMap<ServiceManager, Vec<&ServiceEnable>> = BTreeMap::new();
    print_header("Service enables");

    for service in services.iter() {
        let list = sorted_changes.entry(service.manager).or_default();
        list.push(service);
    }

    for (manager, services) in sorted_changes {
        println!("{}:", manager.to_string().bold());
        for service in services {
            if service.start {
                println!("  {} {} (will be started)", "+".green(), service.name);
            } else {
                println!("  {} {}", "+".green(), service.name);
            }
        }
    }
}

pub fn print_service_disables(services: &[ServiceDisable]) {
    let mut sorted_changes: BTreeMap<ServiceManager, Vec<&ServiceDisable>> = BTreeMap::new();
    print_header("Service disables");

    for service in services.iter() {
        let list = sorted_changes.entry(service.manager).or_default();
        list.push(service);
    }

    for (manager, services) in sorted_changes {
        println!("{}:", manager.to_string().bold());
        for service in services {
            println!("  {} {} (will be stopped)", "-".red(), service.name);
        }
    }
}

/// Print a list of path operations, including content diffs for
/// modifications (via an external diff tool).
pub fn print_path_changes(changes: &[PathOperation], config: &Configuration) -> Result<()> {
    let mut change_iter = changes.iter().peekable();
    print_header("File changes");

    while let Some(op) = change_iter.next() {
        match op {
            PathOperation::File(op) => match op {
                FileOperation::Create {
                    path,
                    content,
                    mode,
                    owner,
                    group,
                } => {
                    println!(
                        "{} {}:      {}",
                        "New".green().bold(),
                        "file".bold(),
                        style_path(path)
                    );

                    let mut table = Table::new();
                    add_table_row(&mut table, "Mod", &format!("{mode:#o}"));

                    // Don't show user/group when it's the default user/group.
                    if *owner != *CURRENT_USER {
                        add_table_row(&mut table, "Owner", owner);
                    }
                    if *group != *CURRENT_GROUP {
                        add_table_row(&mut table, "Group", group);
                    }
                    if let FileContent::Binary(bytes) = content {
                        add_table_row(
                            &mut table,
                            "Content",
                            &format!("binary, {} bytes", bytes.len()),
                        );
                    }
                    print_table(table);
                }
                FileOperation::Modify {
                    path,
                    content,
                    mode,
                    owner,
                    group,
                } => {
                    println!(
                        "{} {}: {}",
                        "Modifying".yellow().bold(),
                        "file".bold(),
                        path.to_string_lossy(),
                    );

                    let mut table = Table::new();

                    if let Some(mode) = mode {
                        add_table_row(&mut table, "Mod", &format!("{mode:#o}"));
                    }

                    if let Some(owner) = owner {
                        add_table_row(&mut table, "Owner", owner);
                    }
                    if let Some(group) = group {
                        add_table_row(&mut table, "Group", group);
                    }
                    if !table.is_empty() {
                        print_table(table);
                    }

                    if let Some(new_content) = content {
                        // Diff direction: actual file (old) -> desired content (new).
                        print_content_diff(config, path, new_content)?;
                    }
                }
                FileOperation::Cleanup { path } => {
                    println!(
                        "{} {}: {}",
                        "Deleting".red().bold(),
                        "file".bold(),
                        path.to_string_lossy(),
                    );
                }
                FileOperation::Conflict { path, found } => {
                    println!(
                        "{} {}: {}",
                        "Removing".red().bold(),
                        format!("conflicting {found}").bold(),
                        path.to_string_lossy(),
                    );
                }
            },
            PathOperation::Directory(op) => match op {
                DirectoryOperation::Create {
                    path,
                    mode,
                    owner,
                    group,
                } => {
                    println!(
                        "{} {}: {}",
                        "New".green().bold(),
                        "directory".bold(),
                        path.to_string_lossy(),
                    );

                    let mut table = Table::new();
                    add_table_row(&mut table, "Mod", &format!("{mode:#o}"));

                    // Don't show user/group when it's the default user/group.
                    if *owner != *CURRENT_USER {
                        add_table_row(&mut table, "Owner", owner);
                    }
                    if *group != *CURRENT_GROUP {
                        add_table_row(&mut table, "Group", group);
                    }
                    print_table(table);
                }
                DirectoryOperation::Modify {
                    path,
                    mode,
                    owner,
                    group,
                } => {
                    println!(
                        "{} {}: {}",
                        "Modifying".yellow().bold(),
                        "directory".bold(),
                        path.to_string_lossy(),
                    );

                    let mut table = Table::new();

                    if let Some(mode) = mode {
                        add_table_row(&mut table, "Mod", &format!("{mode:#o}"));
                    }

                    if let Some(owner) = owner {
                        add_table_row(&mut table, "Owner", owner);
                    }
                    if let Some(group) = group {
                        add_table_row(&mut table, "Group", group);
                    }
                    if !table.is_empty() {
                        print_table(table);
                    }
                }
                DirectoryOperation::Cleanup { path } => {
                    println!(
                        "{} {}: {}",
                        "Deleting".red().bold(),
                        "directory".bold(),
                        path.to_string_lossy(),
                    );
                }
                DirectoryOperation::Conflict { path, found } => {
                    println!(
                        "{} {}: {}",
                        "Removing".red().bold(),
                        format!("conflicting {found}").bold(),
                        path.to_string_lossy(),
                    );
                    println!("  (fails if the directory isn't empty)");
                }
            },
        }

        // Print a delimiter between change entries
        if change_iter.peek().is_some() {
            println!("{}", "              ".underlined());
        }
    }
    Ok(())
}

/// Print everything that changed on the system since the last deployment.
/// Diff direction is deployed (old) -> actual (new): the diff shows what the
/// user changed on their system.
pub fn print_drift(drift: &Drift, config: &Configuration) -> Result<()> {
    print_header("Drift on the system since the last deploy");

    for PathChange { path, change } in &drift.changed_paths {
        match change {
            PathChangeKind::FileTypeChanged { deployed, actual } => {
                println!(
                    "{} {}: was deployed as {}, is now a {}",
                    "Replaced".yellow().bold(),
                    path.to_string_lossy(),
                    deployed.to_string().bold(),
                    actual.to_string().bold(),
                );
            }
            PathChangeKind::Modified {
                content,
                mode,
                owner,
                group,
            } => {
                println!("{} {}", "Modified".yellow().bold(), path.to_string_lossy());

                let mut table = Table::new();
                if let Some((deployed, actual)) = mode {
                    add_table_row(&mut table, "Mod", &format!("{deployed:#o} -> {actual:#o}"));
                }
                if let Some((deployed, actual)) = owner {
                    add_table_row(&mut table, "Owner", &format!("{deployed} -> {actual}"));
                }
                if let Some((deployed, actual)) = group {
                    add_table_row(&mut table, "Group", &format!("{deployed} -> {actual}"));
                }
                if !table.is_empty() {
                    print_table(table);
                }

                if let Some(content_change) = content {
                    // Write the *deployed* content to a temp file and diff it
                    // against the actual file: deployed -> actual.
                    print_diff_against_temp(config, &content_change.deployed, path)?;
                }
            }
        }
    }

    for path in &drift.deleted_paths {
        println!(
            "{} {}: was deployed, no longer exists",
            "Deleted".red().bold(),
            path.to_string_lossy()
        );
    }

    for (manager, package) in &drift.removed_packages {
        println!(
            "{} package {} ({manager}): still configured, will be re-installed",
            "Removed".red().bold(),
            package.clone().bold(),
        );
    }

    for (manager, service) in &drift.disabled_services {
        println!(
            "{} service {} ({manager}): still configured, will be re-enabled",
            "Disabled".red().bold(),
            service.clone().bold(),
        );
    }

    Ok(())
}

fn print_header(header: &str) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
    table.add_row(vec![Cell::new(header).add_attribute(Attribute::Bold)]);

    // Center the header
    let column = table.column_mut(0).unwrap();
    column.set_cell_alignment(CellAlignment::Center);

    table.load_preset(presets::UTF8_FULL);
    println!("{table}\n");
}

fn style_path(path: &Path) -> String {
    let mut path = path.to_path_buf();
    // Get the filename
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    // Remove the filename from the path.
    path.pop();

    format!("{}/{}", path.to_string_lossy(), filename.yellow())
}

fn add_table_row(table: &mut Table, name: &str, value: &str) {
    table.add_row(vec![
        Cell::new(name).add_attribute(Attribute::Bold),
        Cell::new(value),
    ]);
}

fn print_table(mut table: Table) {
    table.load_preset(presets::NOTHING);
    {
        let mut columns = table.column_iter_mut().collect::<Vec<&mut Column>>();
        columns[0].set_padding((2, 0));
    }

    println!("{table}");
}

/// Show the content diff `actual file -> desired content` for a deploy Modify.
fn print_content_diff(config: &Configuration, path: &Path, desired: &FileContent) -> Result<()> {
    match desired {
        FileContent::Binary(bytes) => {
            println!("  Binary content changed ({} bytes)", bytes.len());
            Ok(())
        }
        FileContent::Text(_) => {
            let temp_path = write_temp_diff_file(config, desired)?;
            print_file_diff(path, &temp_path)
        }
    }
}

/// Show the content diff `deployed content -> actual file` for drift reporting.
fn print_diff_against_temp(
    config: &Configuration,
    deployed: &FileContent,
    actual_path: &Path,
) -> Result<()> {
    match deployed {
        FileContent::Binary(_) => {
            println!("  Binary content changed");
            Ok(())
        }
        FileContent::Text(_) => {
            let temp_path = write_temp_diff_file(config, deployed)?;
            print_file_diff(&temp_path, actual_path)
        }
    }
}

/// Write content to a temporary file in the runtime dir, so external diff
/// tools can be pointed at it.
fn write_temp_diff_file(
    config: &Configuration,
    content: &FileContent,
) -> Result<std::path::PathBuf> {
    std::fs::create_dir_all(&config.runtime_dir).map_err(|err| {
        Error::IoPathString(
            config.runtime_dir.clone(),
            "creating runtime directory".to_string(),
            err,
        )
    })?;

    let temp_path = config.runtime_dir.join("bois_diff_file");
    let mut temporary_file = File::create(&temp_path)
        .map_err(|err| Error::IoPath(temp_path.clone(), "opening temporary diff file.", err))?;
    temporary_file
        .write_all(content.bytes())
        .map_err(|err| Error::IoPath(temp_path.clone(), "writing to temporary diff file.", err))?;

    Ok(temp_path)
}

/// Run an external diff tool on two paths.
///
/// Degrades gracefully when the tool isn't installed.
fn print_file_diff(original: &Path, new: &Path) -> Result<()> {
    let args = vec![
        original.to_string_lossy().to_string(),
        new.to_string_lossy().to_string(),
    ];
    let output = match Command::new("delta").args(&args).output() {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            println!("  (install `delta` to see content diffs)");
            return Ok(());
        }
        Err(err) => return Err(Error::Process("delta", err).into()),
    };

    // delta exits 0 for no differences and 1 when differences were found.
    let stdout = String::from_utf8_lossy(&output.stdout);
    match output.status.code() {
        Some(0) | Some(1) => println!("{stdout}"),
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Failed to run diff command ({args:?}): \nstdout:\n{stdout}\nstderr:\n{stderr}"
            );
        }
    }

    Ok(())
}
