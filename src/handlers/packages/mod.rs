use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use strum_macros::Display;

use crate::changeset::PackageOperation;

pub mod pacman;

#[derive(Hash, PartialEq, Eq, Clone, Debug, Display, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PackageManager {
    Pacman,
    Paru,
    Apt,
}

pub fn handle_package_operation(op: PackageOperation) -> Result<()> {
    match op {
        PackageOperation::Add { manager, name } => match manager {
            PackageManager::Pacman => todo!(),
            PackageManager::Paru => todo!(),
            PackageManager::Apt => todo!(),
        },
        PackageOperation::Remove { manager, name } => match manager {
            PackageManager::Pacman => todo!(),
            PackageManager::Paru => todo!(),
            PackageManager::Apt => todo!(),
        },
    }
}
