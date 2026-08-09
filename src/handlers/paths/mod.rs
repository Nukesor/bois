use anyhow::Result;
use directory::{
    cleanup_directory,
    create_directory,
    modify_directory,
    remove_conflicting_directory,
};
use file::{create_file, modify_file, remove_file};

mod directory;
mod file;

use crate::changeset::{DirectoryOperation, FileOperation, PathOperation};

/// Execute a full set of path operations.
///
/// After this function has run, the paths should be in their desired state.
/// Any error in here is considered critical, as continuing could lead to
/// dependency problems and broken state on the system.
pub fn execute_path_operations(operations: &[PathOperation]) -> Result<()> {
    for op in operations.iter() {
        execute_path_operation(op)?
    }

    Ok(())
}

fn execute_path_operation(op: &PathOperation) -> Result<()> {
    match op {
        PathOperation::File(op) => match op {
            FileOperation::Create {
                path,
                content,
                mode,
                owner,
                group,
            } => create_file(path, content.bytes(), *mode, owner, group),
            FileOperation::Modify {
                path,
                content,
                mode,
                owner,
                group,
            } => modify_file(
                path,
                &content.as_ref().map(|content| content.bytes().to_vec()),
                mode,
                owner,
                group,
            ),
            FileOperation::Cleanup { path } | FileOperation::Conflict { path, .. } => {
                remove_file(path)
            }
        },
        PathOperation::Directory(op) => match op {
            DirectoryOperation::Create {
                path,
                mode,
                owner,
                group,
            } => create_directory(path, *mode, owner, group),
            DirectoryOperation::Modify {
                path,
                mode,
                owner,
                group,
            } => modify_directory(path, mode, owner, group),
            DirectoryOperation::Cleanup { path } => cleanup_directory(path),
            DirectoryOperation::Conflict { path, .. } => remove_conflicting_directory(path),
        },
    }
}
