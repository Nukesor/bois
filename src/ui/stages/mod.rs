use std::iter::repeat_n;

use crossterm::{style::Stylize, terminal};

pub mod drift;

/// Print a header for a stage section in the non-TUI CLI.
///
/// Example:
/// "── Deteted Drift ────────────────────────────"
fn print_header(text: &str) {
    let mut header = format!("── {} ", text.bold());

    if let Ok(size) = terminal::window_size() {
        let remaining = size.width - header.len() as u16;
        header.extend(repeat_n("─", remaining.into()));
    };

    println!("{header}\n");
}
