#![allow(dead_code)]
use std::path::Path;

use crate::ui::theme::Styled;

mod diff;
pub mod stages;
pub mod style;
pub mod theme;

/// Display a path with only its last component styled.
pub(crate) fn style_path<F>(path: &Path, style: F) -> String
where
    F: FnOnce(String) -> Styled<String>,
{
    let mut path = path.to_path_buf();
    // Get the filename
    let Some(filename) = path.file_name() else {
        return path.to_string_lossy().into_owned();
    };
    let filename = filename.to_string_lossy().to_string();
    // Remove the filename from the path.
    path.pop();

    format!("{}/{}", path.to_string_lossy(), style(filename))
}
