use std::collections::HashSet;

use anyhow::Result;

use crate::state::PackageManager;

pub mod pacman;
pub mod paru;

/// Return the set of all explicitly installed groups on the system.
pub fn get_detected_groups(manager: PackageManager) -> Result<HashSet<String>> {
    match manager {
        PackageManager::Pacman => pacman::detect_installed_groups(),
        PackageManager::Paru => Ok(HashSet::new()),
        PackageManager::Apt => todo!(),
    }
}
