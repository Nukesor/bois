use std::{
    collections::BTreeMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Result};
use comfy_table::{presets, Attribute, Cell, CellAlignment, Column, ContentArrangement, Table};
use crossterm::style::Stylize;

use crate::{
    changeset::{Change, Changeset, PackageOperation, PathOperation},
    constants::{CURRENT_GROUP, CURRENT_USER},
    error::Error,
    handlers::packages::PackageManager,
};

pub fn print_package_removals(changes: &Changeset) {
    let mut sorted_changes: BTreeMap<PackageManager, Vec<String>> = BTreeMap::new();
    print_header("Package removals");

    for change in changes.iter() {
        if let Change::PackageChange(PackageOperation::Remove { manager, name }) = change {
            let list = sorted_changes.entry(*manager).or_default();
            list.push(name.clone());
        }
    }

    for (manager, packages) in sorted_changes {
        println!("{}:", manager.to_string().bold());
        for package in packages {
            println!("  {} {package}", "-".red());
        }
    }
}

pub fn print_package_additions(changes: &Changeset) {
    let mut sorted_changes: BTreeMap<PackageManager, Vec<String>> = BTreeMap::new();
    print_header("Package additions");

    for change in changes.iter() {
        if let Change::PackageChange(PackageOperation::Add { manager, name }) = change {
            let list = sorted_changes.entry(*manager).or_default();
            list.push(name.clone());
        }
    }

    for (manager, packages) in sorted_changes {
        println!("{}:", manager.to_string().bold());
        for package in packages {
            println!("  {} {package}", "+".green());
        }
    }
}

pub fn print_path_changes(changes: &Changeset) -> Result<()> {
    let mut change_iter = changes.iter().peekable();
    print_header("File changes");

    while let Some(change) = change_iter.next() {
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
                    println!(
                        "{} {}:      {}",
                        "New".green().bold(),
                        "file".bold(),
                        style_path(path)
                    );

                    let mut table = Table::new();
                    add_table_row(&mut table, "Mod", &format!("{permissions:#o}"));

                    // Don't show user/group when it's the default user/group.
                    if *owner != *CURRENT_USER {
                        add_table_row(&mut table, "Owner", owner);
                    }
                    if *group != *CURRENT_GROUP {
                        add_table_row(&mut table, "Group", group);
                    }
                    print_table(table);
                }
                crate::changeset::FileOperation::Modify {
                    path,
                    content,
                    permissions,
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

                    if let Some(permissions) = permissions {
                        add_table_row(&mut table, "Mod", &format!("{permissions:#o}"));
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
                        let temp_path = PathBuf::from("/var/lib/bois/").join("bois_new_file");

                        // Write file to a temporary file in the user's runtime directory.
                        // That way, we can diff the file with external tools.
                        {
                            if temp_path.exists() {
                                std::fs::remove_file(&temp_path).map_err(|err| {
                                    Error::IoPath(temp_path.clone(), "removing old temp file.", err)
                                })?;
                            };

                            let mut temporary_file = File::create(&temp_path).map_err(|err| {
                                Error::IoPath(
                                    temp_path.clone(),
                                    "opening temporary diff file.",
                                    err,
                                )
                            })?;

                            temporary_file.write_all(new_content).map_err(|err| {
                                Error::IoPath(
                                    temp_path.clone(),
                                    "writing to temporary diff file.",
                                    err,
                                )
                            })?;
                        }

                        print_file_diff(path, &temp_path)?;
                    }
                }
                crate::changeset::FileOperation::Delete { .. } => continue,
            },
            PathOperation::Directory(op) => match op {
                crate::changeset::DirectoryOperation::Create {
                    path,
                    permissions,
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
                    add_table_row(&mut table, "Mod", &format!("{permissions:#o}"));

                    // Don't show user/group when it's the default user/group.
                    if *owner != *CURRENT_USER {
                        add_table_row(&mut table, "Owner", owner);
                    }
                    if *group != *CURRENT_GROUP {
                        add_table_row(&mut table, "Group", group);
                    }
                    print_table(table);
                }
                crate::changeset::DirectoryOperation::Modify {
                    path,
                    permissions,
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

                    if let Some(permissions) = permissions {
                        add_table_row(&mut table, "Mod", &format!("{permissions:#o}"));
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
                crate::changeset::DirectoryOperation::Delete { .. } => continue,
            },
        }

        // Print a delimiter between change entries
        if change_iter.peek().is_some() {
            println!("{}", "              ".underlined());
        }
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

/// Run an external diff tool on two paths.
fn print_file_diff(original: &Path, new: &Path) -> Result<()> {
    let output = Command::new("delta")
        .args(vec![
            original.to_string_lossy().to_string(),
            new.to_string_lossy().to_string(),
        ])
        .output()
        .map_err(|err| Error::Process("running", err))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let code = output.status.code();
    if code.is_none() {
        bail!("Failed to run diff command: \nstdout:\n{stdout}\nstderr:\n{stderr}");
    } else if let Some(code) = code {
        if code != 1 {
            bail!("Failed to run diff command: \nstdout:\n{stdout}\nstderr:\n{stderr}");
        }
    }

    println!("{stdout}");

    Ok(())
}
