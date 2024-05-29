use anyhow::Result;

use crate::{changeset::Change, system_state::SystemState};

use self::packages::handle_package_operation;

pub mod packages;
pub mod services;

/// Execute a full set of changes.
/// After this function has run, the system should be in its desired state, any errors in here are
/// to be considered critical.
/// Continueing on error could lead to dependency problems and further broken state in the system.
pub fn handle_changeset(system_state: &mut SystemState, changeset: &Vec<Change>) -> Result<()> {
    for change in changeset.iter() {
        match change {
            Change::PackageChange(op) => handle_package_operation(system_state, op)?,
        }
    }

    Ok(())
}
