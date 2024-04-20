use std::collections::HashSet;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use strum_macros::Display;

use crate::changeset::PackageOperation;

pub mod pacman;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Display, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PackageManager {
    Pacman,
    Paru,
    Apt,
}

pub fn handle_package_operation(op: &PackageOperation) -> Result<()> {
    match op {
        PackageOperation::Add { manager, name } => match manager {
            PackageManager::Pacman => pacman::install_package(name),
            PackageManager::Paru => todo!(),
            PackageManager::Apt => todo!(),
        },
        PackageOperation::Remove { manager, name } => match manager {
            PackageManager::Pacman => pacman::uninstall_package(name),
            PackageManager::Paru => todo!(),
            PackageManager::Apt => todo!(),
        },
    }
}

/// Return the set of all explicitly installed packages on the system.
pub fn get_installed_packages(manager: PackageManager) -> Result<HashSet<String>> {
    match manager {
        PackageManager::Pacman => pacman::get_installed_packages(),
        PackageManager::Paru => todo!(),
        PackageManager::Apt => todo!(),
    }
}
