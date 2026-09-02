//! All logic for printing output during in the non-TUI of bois.

use std::iter::repeat_n;

use comfy_table::{Cell, Table, presets};
use crossterm::terminal;

use crate::{
    changeset::{DirectoryOperation, FileOperation, FileType, PathOperation},
    constants::{CURRENT_GROUP, CURRENT_USER},
    state::path::FileContent,
    ui::{diff::Diff, style_path, theme::Stylize},
};

pub mod cleanup;
pub mod deploy;
pub mod drift;

/// Print a header for a stage section in the non-TUI CLI.
///
/// Example:
/// "── Detected Drift ────────────────────────────"
fn print_header(text: &str) {
    let mut header = format!("── {} ", text.bold());

    if let Ok((cols, _)) = terminal::size() {
        let remaining = (cols as usize)
            .checked_sub(header.len())
            .unwrap_or(header.len());
        header.extend(repeat_n("─", remaining));
    };

    println!("{header}\n");
}

/// A row of a stage's path table.
///
/// Only the properties that're relevant for the row's operation are set.
/// Property columns without a single entry are hidden from the table.
#[derive(Default)]
struct PathRow {
    path: String,
    mode: Option<String>,
    group: Option<String>,
    user: Option<String>,
    content: Option<String>,
}

/// A rather generic helper to print a path information table during a deploy.
///
/// Each row represents a changed path and may have colums for mode, owner, group or changed
/// content.
fn print_path_table(path_title: String, rows: &[PathRow]) {
    // The optional property columns: header + value getter.
    type PropertyColumn = (&'static str, fn(&PathRow) -> Option<&String>);

    // Generic accessor for the different columns
    let property_columns: [PropertyColumn; 4] = [
        ("Content", |row| row.content.as_ref()),
        ("Mod", |row| row.mode.as_ref()),
        ("Group", |row| row.group.as_ref()),
        ("User", |row| row.user.as_ref()),
    ];

    // Figure out which of the property_columns has at least one entry.
    let columns: Vec<_> = property_columns
        .into_iter()
        .filter(|(_, value)| rows.iter().any(|row| value(row).is_some()))
        .collect();

    let mut table = Table::new();
    table.load_style(presets::NOTHING);

    let mut header = vec![Cell::new(path_title.bold())];
    header.extend(columns.iter().map(|(name, _)| Cell::new(name.bold())));
    table.set_header(header);

    // Build the rows for all colums that have at least one value.
    for row in rows {
        let mut cells = vec![Cell::new(&row.path)];
        cells.extend(
            columns
                .iter()
                .map(|(_, value)| Cell::new(value(row).map(String::as_str).unwrap_or_default())),
        );
        table.add_row(cells);
    }

    // Give the property columns some extra spacing to their left neighbor.
    // The default padding is quite crowded.
    for column in table.column_iter_mut().skip(1) {
        column.set_padding((3, 1));
    }

    println!("{table}");
}

/// Build the path table row for a single path operation.
///
/// This is shared between the cleanup and deploy stages.
fn path_operation_row(op: &PathOperation) -> PathRow {
    match op {
        PathOperation::File(op) => match op {
            FileOperation::Create {
                path,
                content,
                mode,
                owner,
                group,
            } => {
                let mut row = creation_row(path, &FileType::File, *mode, owner, group);
                if let FileContent::Binary(bytes) = content {
                    row.content = Some(format!(
                        "binary · {}",
                        Diff::human_bytes(bytes.len() as u64)
                    ));
                }
                row
            }
            FileOperation::Modify {
                path,
                content,
                mode,
                owner,
                group,
            } => {
                let mut row = modification_row(path, &FileType::File, mode, owner, group);
                row.content = content.as_ref().map(|_| "changed".change().to_string());
                row
            }
            FileOperation::Cleanup { path } => deletion_row(path, &FileType::File),
            FileOperation::Conflict { path, found } => conflict_row(path, found, false),
        },
        PathOperation::Directory(op) => match op {
            DirectoryOperation::Create {
                path,
                mode,
                owner,
                group,
            } => creation_row(path, &FileType::Directory, *mode, owner, group),
            DirectoryOperation::Modify {
                path,
                mode,
                owner,
                group,
            } => modification_row(path, &FileType::Directory, mode, owner, group),
            DirectoryOperation::Cleanup { path } => deletion_row(path, &FileType::Directory),
            DirectoryOperation::Conflict { path, found } => conflict_row(path, found, true),
        },
    }
}

/// The row for a path that'll be newly created.
fn creation_row(
    path: &std::path::Path,
    filetype: &FileType,
    mode: u32,
    owner: &str,
    group: &str,
) -> PathRow {
    let mut row = PathRow {
        path: format!(
            "{} {}",
            filetype.emoji(),
            style_path(path, |name| name.addition().bold())
        ),
        content: Some("new".addition().to_string()),
        mode: Some(format!("{mode:#o}").addition().to_string()),
        ..Default::default()
    };

    // Don't show user/group when it's the default user/group.
    if *owner != *CURRENT_USER {
        row.user = Some(owner.addition().to_string());
    }
    if *group != *CURRENT_GROUP {
        row.group = Some(group.addition().to_string());
    }

    row
}

/// The row for a path whose properties will be modified.
/// The old values aren't known here, so the cells show `→ new-value`.
fn modification_row(
    path: &std::path::Path,
    filetype: &FileType,
    mode: &Option<u32>,
    owner: &Option<String>,
    group: &Option<String>,
) -> PathRow {
    PathRow {
        path: format!(
            "{} {}",
            filetype.emoji(),
            style_path(path, |name| name.highlight().bold())
        ),
        mode: mode.map(|mode| format!("→ {}", format!("{mode:#o}").change())),
        user: owner.as_ref().map(|owner| format!("→ {}", owner.change())),
        group: group.as_ref().map(|group| format!("→ {}", group.change())),
        ..Default::default()
    }
}

/// The row for a path that'll be deleted.
fn deletion_row(path: &std::path::Path, filetype: &FileType) -> PathRow {
    PathRow {
        path: format!(
            "{} {}",
            filetype.emoji(),
            style_path(path, |name| name.removal().bold())
        ),
        ..Default::default()
    }
}

/// The row for a conflicting unmanaged entry that'll be removed.
fn conflict_row(path: &std::path::Path, found: &FileType, directory_target: bool) -> PathRow {
    let mut content = format!("remove conflicting {found}").removal().to_string();
    if directory_target {
        content.push_str(" (fails if the directory isn't empty)");
    }

    PathRow {
        path: format!(
            "{} {}",
            found.emoji(),
            style_path(path, |name| name.removal().bold())
        ),
        content: Some(content),
        ..Default::default()
    }
}
