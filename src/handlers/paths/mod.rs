use anyhow::Result;

use crate::{changeset::PathOperation, system_state::SystemState};

pub fn handle_path_operation(_system_state: &mut SystemState, op: &PathOperation) -> Result<()> {
    match op {
        PathOperation::File(op) => match op {
            crate::changeset::FileOperation::Create { .. } => todo!(),
            crate::changeset::FileOperation::Modify { .. } => todo!(),
            crate::changeset::FileOperation::Delete => todo!(),
        },
        PathOperation::Directory(op) => match op {
            crate::changeset::DirectoryOperation::Create { .. } => todo!(),
            crate::changeset::DirectoryOperation::Modify { .. } => todo!(),
            crate::changeset::DirectoryOperation::Delete { .. } => todo!(),
        },
    }
}
