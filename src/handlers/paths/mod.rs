use anyhow::Result;
use directory::{create_directory, modify_directory, remove_directory};
use file::{create_file, modify_file, remove_file};

mod directory;
mod file;

use crate::{changeset::PathOperation, system_state::SystemState};

/// Execute a full set of changes.
/// After this function has run, the system should be in its desired state, any errors in here are
/// to be considered critical.
/// Continueing on error could lead to dependency problems and further broken state in the system.
pub fn handle_path_operations(
    system_state: &mut SystemState,
    operations: &[PathOperation],
) -> Result<()> {
    for op in operations.iter() {
        handle_path_operation(system_state, op)?
    }

    Ok(())
}

fn handle_path_operation(_system_state: &mut SystemState, op: &PathOperation) -> Result<()> {
    match op {
        PathOperation::File(op) => match op {
            crate::changeset::FileOperation::Create {
                path,
                content,
                mode,
                owner,
                group,
            } => create_file(path, content, mode, owner, group),
            crate::changeset::FileOperation::Modify {
                path,
                content,
                mode,
                owner,
                group,
            } => modify_file(path, content, mode, owner, group),
            crate::changeset::FileOperation::Delete { path } => remove_file(path),
        },
        PathOperation::Directory(op) => match op {
            crate::changeset::DirectoryOperation::Create {
                path,
                mode,
                owner,
                group,
            } => create_directory(path, mode, owner, group),
            crate::changeset::DirectoryOperation::Modify {
                path,
                mode,
                owner,
                group,
            } => modify_directory(path, mode, owner, group),
            crate::changeset::DirectoryOperation::Delete { path } => remove_directory(path),
        },
    }
}
