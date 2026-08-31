use std::num::ParseIntError;

use anyhow::Result;
use dialoguer::{Input, theme::ColorfulTheme};

use super::print_header;
use crate::{
    changeset::{Drift, PathChange, PathChangeKind},
    config::bois::Configuration,
    error::Error,
    ui::{diff::Diff, theme::Stylize},
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
        println!("Paths:");

        for (index, PathChange { path, change }) in drift.changed_paths.iter().enumerate() {
            match change {
                PathChangeKind::FileTypeChanged { deployed, actual } => {
                    println!(
                        "{}: Filetype {} → {}",
                        path.display().highlight().bold(),
                        deployed.bold(),
                        actual.bold(),
                    );
                }
                PathChangeKind::Modified {
                    content,
                    mode,
                    owner,
                    group,
                } => {
                    let mut message = format!("{} ", path.display().highlight().bold());

                    if let Some((deployed, actual)) = mode {
                        message.push_str(&format!("mod {deployed:#o} → {actual:#o}, "));
                    }
                    if let Some((deployed, actual)) = owner {
                        message.push_str(&format!("owner {deployed} → {actual}, "));
                    }
                    if let Some((deployed, actual)) = group {
                        message.push_str(&format!("group {deployed} → {actual}, "));
                    }
                    if content.is_some() {
                        message.push_str("content changed");
                        content_changes.push(index);
                    }

                    println!("{}", message.strip_suffix(", ").unwrap_or(&message));

                    // In a dry-run mode, we also always print diffs.
                    if dry_run && let Some(change) = content {
                        let diff = Diff::for_drift(&change.deployed, &change.actual);
                        println!("{}", diff.format());
                    }
                }
            }
        }

        for path in &drift.deleted_paths {
            println!("{}: Deleted", path.display().removal().bold());
        }
    }

    if !dry_run {
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
        // Unwrap, as we just know these indices exist.
        let change = drift.changed_paths.get(*id).unwrap();
        let PathChangeKind::Modified {
            content: Some(content_change),
            ..
        } = &change.change
        else {
            unreachable!();
        };

        let diff = Diff::for_drift(&content_change.deployed, &content_change.actual);

        println!("Changes for path {}", change.path.to_string_lossy());
        println!("{}", diff.format());
    }

    // Now that we presented the diffs, show the prompt again.
    handle_prompt(drift, content_changes)?;

    Ok(())
}
