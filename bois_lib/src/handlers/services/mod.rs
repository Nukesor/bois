use serde::{Deserialize, Serialize};
use strum::Display;

pub mod systemd;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Display, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ServiceManager {
    Systemd,
}
