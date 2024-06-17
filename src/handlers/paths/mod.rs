use anyhow::Result;
use directory::{create_file, modify_file};

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
            crate::changeset::FileOperation::Delete { path } => todo!(),
        },
        PathOperation::Directory(op) => match op {
            crate::changeset::DirectoryOperation::Create { .. } => todo!(),
            crate::changeset::DirectoryOperation::Modify { .. } => todo!(),
            crate::changeset::DirectoryOperation::Delete { .. } => todo!(),
        },
    }
}
