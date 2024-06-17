use anyhow::Result;
use directory::{create_directory, modify_directory, remove_directory};
use file::{create_file, modify_file, remove_file};

mod directory;
mod file;

use crate::{changeset::PathOperation, system_state::SystemState};

pub fn handle_path_operation(_system_state: &mut SystemState, op: &PathOperation) -> Result<()> {
    match op {
        PathOperation::File(op) => match op {
            crate::changeset::FileOperation::Create {
                path,
                content,
                permissions,
                owner,
                group,
            } => create_file(path, content, permissions, owner, group),
            crate::changeset::FileOperation::Modify {
                path,
                content,
                permissions,
                owner,
                group,
            } => modify_file(path, content, permissions, owner, group),
            crate::changeset::FileOperation::Delete { path } => remove_file(path),
        },
        PathOperation::Directory(op) => match op {
            crate::changeset::DirectoryOperation::Create {
                path,
                permissions,
                owner,
                group,
            } => create_directory(path, permissions, owner, group),
            crate::changeset::DirectoryOperation::Modify {
                path,
                permissions,
                owner,
                group,
            } => modify_directory(path, permissions, owner, group),
            crate::changeset::DirectoryOperation::Delete { path } => remove_directory(path),
        },
    }
}
