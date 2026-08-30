use std::sync::OnceLock;

use crossterm::style::Color;

/// The global theme: whether styled *stdout* output is enabled, and which
/// palette variant applies. Set once at startup; plain + dark until then.
static STYLE: OnceLock<OutputStyler> = OnceLock::new();

/// Set the color policy and palette variant globally.
///
/// Called once at startup after the color policy (TTY, `NO_COLOR`, `--color`)
/// and the theme (`--theme`, background detection) have been resolved. Later
/// calls have no effect.
pub fn init(styler: OutputStyler) {
    let _ = STYLE.set(styler);
}

/// Whether colored output is enabled. Defaults to plain output until
/// [init] ran.
pub fn color_enabled() -> bool {
    STYLE.get().is_some_and(|styler| styler.enabled)
}

/// Helper struct, which provides all info and helper functions for style.
/// - Enables styles if color mode is 'always', or if color mode is 'auto' and output is a tty.
/// - Using dark colors if dark_mode is enabled
#[derive(Debug, Clone)]
pub struct OutputStyler {
    /// Whether or not ANSI styling is enabled
    pub enabled: bool,
    /// The currently active theme
    pub theme: Theme,

    /// The color palette,
    pub palette: Palette,
}

#[derive(Debug, Clone, Default)]
pub enum Theme {
    #[default]
    None,
    Dark,
    Light,
}

#[derive(Debug, Clone)]
pub struct Palette {
    // General styling
    text: Color,
    highlight: Color,

    // Color palette for diffs
    addition: Color,
    removal: Color,
    change: Color,
    unchanged: Color,

    // Debug info color palette
    help: Color,
    info: Color,
    warning: Color,
    error: Color,
}

impl Palette {
    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::None => unreachable!(),
            Theme::Dark => Self {
                text: todo!(),
                highlight: todo!(),
                addition: todo!(),
                removal: todo!(),
                change: todo!(),
                unchanged: todo!(),
                help: todo!(),
                info: todo!(),
                warning: todo!(),
                error: todo!(),
            },
            Theme::Light => Self {
                text: todo!(),
                highlight: todo!(),
                addition: todo!(),
                removal: todo!(),
                change: todo!(),
                unchanged: todo!(),
                help: todo!(),
                info: todo!(),
                warning: todo!(),
                error: todo!(),
            },
        }
    }
}
