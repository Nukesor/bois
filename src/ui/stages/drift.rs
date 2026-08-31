use std::num::ParseIntError;

use anyhow::Result;
use comfy_table::{Cell, Table, presets};
use dialoguer::{Input, theme::ColorfulTheme};

use super::print_header;
use crate::{
    changeset::{Drift, FileType, PathChange, PathChangeKind},
    config::bois::Configuration,
    error::Error,
    ui::{diff::Diff, style_path, theme::Stylize},
};

/// Print everything that changed on the system since the last run.
/// Diff direction is deployed (old) -> actual (new): the diff shows what the
/// user changed on their system.
pub fn handle_drift(drift: &Drift, _config: &Configuration, dry_run: bool) -> Result<()> {
    print_header("Drift since the last deploy");

    if !drift.removed_packages.is_empty() {
        println!("Uninstalled packages:");
        for (manager, packages) in &drift.removed_packages {
            for package in packages {
                println!("{manager} - {}", package.bold());
            }
        }
    }

    if !drift.disabled_services.is_empty() {
        println!("Disabled services:");
        for (manager, services) in &drift.disabled_services {
            for service in services {
                println!("{manager} - {}", service.bold());
            }
        }
    }

    // Keep track of which path related changes contain diffable changes.
    // We use those for an interactive prompt later on.
    let mut content_changes = Vec::new();

    if !drift.changed_paths.is_empty() || !drift.deleted_paths.is_empty() {
        // When printing the output for the drifted paths, we only want to show the
        // properties that have been changed. As such, we accumulate all rows into
        // a temporary `DriftRow`, which only has optional properties.
        //
        // This allows us to easily check later-on whether there's any row for a given property.
        #[derive(Default)]
        struct DriftRow {
            path: String,
            mode: Option<String>,
            group: Option<String>,
            user: Option<String>,
            content: Option<String>,
        }

        let mut rows = Vec::new();

        for (index, PathChange { path, change }) in drift.changed_paths.iter().enumerate() {
            let filetype = match change {
                PathChangeKind::FileTypeChanged { actual, .. } => actual,
                PathChangeKind::Modified { filetype, .. } => filetype,
            };
            let mut row = DriftRow {
                path: format!(
                    "{} {}",
                    filetype_emoji(filetype),
                    style_path(path, |name| name.highlight().bold())
                ),
                ..Default::default()
            };

            match change {
                PathChangeKind::FileTypeChanged { deployed, actual } => {
                    row.content = Some(format!(
                        "filetype {} → {}",
                        deployed.removal(),
                        actual.change()
                    ));
                }
                PathChangeKind::Modified {
                    content,
                    mode,
                    owner,
                    group,
                    ..
                } => {
                    if let Some((deployed, actual)) = mode {
                        row.mode = Some(format!(
                            "{} → {}",
                            format!("{deployed:#o}").removal(),
                            format!("{actual:#o}").change()
                        ));
                    }
                    if let Some((deployed, actual)) = group {
                        row.group = Some(format!("{} → {}", deployed.removal(), actual.change()));
                    }
                    if let Some((deployed, actual)) = owner {
                        row.user = Some(format!("{} → {}", deployed.removal(), actual.change()));
                    }
                    if content.is_some() {
                        row.content = Some("changed".change().to_string());
                        content_changes.push(index);
                    }
                }
            }

            rows.push(row);
        }

        for path in &drift.deleted_paths {
            rows.push(DriftRow {
                path: style_path(path, |name| name.removal().bold()),
                content: Some("deleted".removal().to_string()),
                ..Default::default()
            });
        }

        // The optional property columns: header + value getter.
        type PropertyColumn = (&'static str, fn(&DriftRow) -> Option<&String>);
        let property_columns: [PropertyColumn; 4] = [
            ("Mod", |row| row.mode.as_ref()),
            ("Group", |row| row.group.as_ref()),
            ("User", |row| row.user.as_ref()),
            ("Content", |row| row.content.as_ref()),
        ];

        // Only show property columns with at least one entry.
        let columns: Vec<_> = property_columns
            .into_iter()
            .filter(|(_, value)| rows.iter().any(|row| value(row).is_some()))
            .collect();

        let mut table = Table::new();
        table.load_style(presets::NOTHING);

        let mut header = vec![Cell::new("Path".bold())];
        header.extend(columns.iter().map(|(name, _)| Cell::new(name.bold())));
        table.set_header(header);

        for row in &rows {
            let mut cells = vec![Cell::new(&row.path)];
            cells.extend(
                columns.iter().map(|(_, value)| {
                    Cell::new(value(row).map(String::as_str).unwrap_or_default())
                }),
            );
            table.add_row(cells);
        }

        // Give the property columns some extra spacing to their left
        // neighbor, the default padding is quite crowded.
        for column in table.column_iter_mut().skip(1) {
            column.set_padding((3, 1));
        }

        println!("{table}");
    }

    // Print the diffs of all changed files after the table during dry runs.
    if dry_run {
        for index in &content_changes {
            // Unwrap, as we know these indices exist.
            let change = drift.changed_paths.get(*index).unwrap();
            let PathChangeKind::Modified {
                content: Some(content_change),
                ..
            } = &change.change
            else {
                unreachable!();
            };

            let diff = Diff::for_drift(&content_change.deployed, &content_change.actual);
            println!(
                "\nChanges for path {}",
                style_path(&change.path, |name| name.highlight())
            );
            println!("{}", diff.format());
        }
    } else {
        handle_prompt(drift, &content_changes)?;
    }

    Ok(())
}

/// Prompt the user whether they want to override the on-system changes.
///
/// The user also has the option to inspect any actual file diffs.
pub fn handle_prompt(drift: &Drift, content_changes: &Vec<usize>) -> Result<()> {
    let mut prompt = format!("Do you want to override those changes? y/{}", "N".bold());

    if !content_changes.is_empty() {
        prompt.push_str(", 'a'\n You can also inspect the diffs: 'a' to show all or ");
        prompt.push_str(
            &content_changes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(", "),
        );
    }

    let input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(&prompt)
        .default("N".to_string())
        .interact_text()?;

    let (valid_input, diffs_to_show) = match input.to_lowercase().trim() {
        "y" | "yes" => return Ok(()),
        "n" | "no" => return Err(Error::Generic("User aborted deploy.".into()).into()),
        "a" | "all" => (true, content_changes.clone()),
        input => 'inner: {
            // Parse the input for multiple ids (delimited with potentially comma, space or both)
            let split = input.split(|c| c == ',' && c == ' ');
            let parsed_values = split
                .into_iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse())
                .collect::<Result<Vec<usize>, ParseIntError>>();

            // If there're any parse values, break early
            let Ok(ids) = parsed_values else {
                break 'inner (false, Vec::new());
            };

            // Otherwise, return the parsed ids, that're actually valid.
            (
                true,
                ids.iter()
                    .filter(|i| content_changes.contains(i))
                    .copied()
                    .collect(),
            )
        }
    };

    // If we got invalid input, just show the prompt again.
    if !valid_input || diffs_to_show.is_empty() {
        handle_prompt(drift, content_changes)?;
    }

    // We got some valid ids, show them:
    for id in content_changes {
        // Unwrap, as we know these indices exist.
        let change = drift.changed_paths.get(*id).unwrap();
        let PathChangeKind::Modified {
            content: Some(content_change),
            ..
        } = &change.change
        else {
            unreachable!();
        };

        let diff = Diff::for_drift(&content_change.deployed, &content_change.actual);

        println!(
            "Changes for path {}",
            style_path(&change.path, |name| name.highlight())
        );
        println!("{}", diff.format());
    }

    // Now that we presented the diffs, show the prompt again.
    handle_prompt(drift, content_changes)?;

    Ok(())
}

/// The emoji marking a path's filetype in the drift table.
fn filetype_emoji(filetype: &FileType) -> &'static str {
    match filetype {
        FileType::File => "📄",
        FileType::Directory => "📁",
        FileType::Symlink => "🔗",
        FileType::Special => "⚙️",
    }
}
