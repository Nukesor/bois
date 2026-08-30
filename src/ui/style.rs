use strum::Display;

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
