use std::iter::repeat_n;

use crossterm::terminal;

use crate::ui::theme::Stylize;

pub mod drift;

/// Print a header for a stage section in the non-TUI CLI.
///
/// Example:
/// "──── Detected Drift ────────────────────────────"
fn print_header(text: &str) {
    let mut header = format!("──── {} ", text.bold());

    if let Ok((cols, _)) = terminal::size() {
        let remaining = (cols as usize)
            .checked_sub(header.len())
            .unwrap_or(header.len());
        header.extend(repeat_n("─", remaining));
    };

    println!("\n{header}");
}
