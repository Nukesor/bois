use anyhow::Result;

use crate::{changeset::Change, system_state::SystemState};
use packages::handle_package_operation;
use paths::handle_path_operation;

pub mod packages;
pub mod paths;
pub mod services;

/// Execute a full set of changes.
/// After this function has run, the system should be in its desired state, any errors in here are
/// to be considered critical.
/// Continueing on error could lead to dependency problems and further broken state in the system.
pub fn handle_changeset(system_state: &mut SystemState, changeset: &[Change]) -> Result<()> {
    for change in changeset.iter() {
        match change {
            Change::PackageChange(op) => handle_package_operation(system_state, op)?,
            Change::PathChange(op) => handle_path_operation(system_state, op)?,
        }
    }

    Ok(())
}
