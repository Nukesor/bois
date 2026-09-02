use strum::Display;

use crate::ui::theme::Stylize;

/// The execution status of an item.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum Status {
    #[strum(serialize = "✓")]
    Applied,
    #[strum(serialize = "✗")]
    Failed,
    #[strum(serialize = "»")]
    Skipped,
    #[strum(serialize = "●")]
    Kept,
    #[strum(serialize = "·")]
    Pending,
}

impl Status {
    pub fn styled(&self) -> String {
        match &self {
            Status::Applied => Status::Applied.addition().to_string(),
            Status::Failed => Status::Failed.removal().to_string(),
            Status::Skipped => Status::Skipped.to_string(),
            Status::Kept => Status::Kept.to_string(),
            Status::Pending => Status::Pending.to_string(),
        }
    }
}
