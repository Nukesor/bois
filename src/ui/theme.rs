//! Styling of user-facing text: the global color policy, the color palette
//! and the [Stylize] trait that applies both.
//!
//! ```ignore
//! use crate::ui::theme::Stylize;
//!
//! println!("{} {}", "New".addition().bold(), path.display().highlight());
//! ```
//!
//! Styling is applied lazily when the [Styled] value is formatted: if styled
//! output is disabled at that point, the plain text is emitted. Palette roles
//! are resolved when the role method is called, so a palette change (theme
//! switch, user configuration) affects everything styled afterwards.

use std::{
    fmt::{self, Display, Formatter},
    sync::{PoisonError, RwLock},
};

use crossterm::style::{Attribute, Color, ContentStyle};

/// The global style: whether styled *stdout* output is enabled, and which
/// palette applies. Always readable; plain + dark until [set] ran.
static STYLE: RwLock<Style> = RwLock::new(Style::DEFAULT);

/// The color policy and palette for all styled output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    /// Whether ANSI styling is emitted at all.
    /// Resolved from the TTY state, `NO_COLOR` and `--color`.
    pub enabled: bool,
    /// The active palette.
    pub palette: Palette,
}

impl Style {
    /// Plain output, dark palette. The state before [set] ran.
    pub const DEFAULT: Style = Style {
        enabled: false,
        palette: Palette::DARK,
    };
}

impl Default for Style {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Set the global style. May be called any number of times, e.g. to switch
/// the palette at runtime.
pub fn set(style: Style) {
    // The whole crate gates styling via [enabled], so crossterm's own gate
    // (`NO_COLOR`) must not interfere: with it active, crossterm emits
    // truncated escape sequences (`\x1b[m`) instead of plain text.
    crossterm::style::force_color_output(true);
    *STYLE.write().unwrap_or_else(PoisonError::into_inner) = style;
}

/// Replace only the active palette.
pub fn set_palette(palette: Palette) {
    STYLE
        .write()
        .unwrap_or_else(PoisonError::into_inner)
        .palette = palette;
}

/// A snapshot of the current global style.
pub fn current() -> Style {
    *STYLE.read().unwrap_or_else(PoisonError::into_inner)
}

/// Whether styled output is enabled.
pub fn enabled() -> bool {
    current().enabled
}

/// A snapshot of the active palette.
pub fn palette() -> Palette {
    current().palette
}

/// The theme variant of the terminal background, used to pick a palette.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

/// The colors used for all styled output, keyed by semantic role.
///
/// Both presets are based on [gruvbox](https://github.com/morhetz/gruvbox):
/// the dark palette uses gruvbox's bright colors, the light palette its faded
/// colors, which are the respective high-contrast variants on each background.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    // General styling
    pub text: Color,
    pub highlight: Color,

    // Color palette for diffs
    pub addition: Color,
    pub removal: Color,
    pub change: Color,
    pub unchanged: Color,

    // Debug info color palette
    pub help: Color,
    pub info: Color,
    pub warning: Color,
    pub error: Color,
}

/// Shorthand for building a truecolor [Color].
const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

impl Palette {
    /// Gruvbox dark.
    pub const DARK: Palette = Palette {
        text: rgb(0xeb, 0xdb, 0xb2),      // fg1
        highlight: rgb(0xfa, 0xbd, 0x2f), // bright yellow

        addition: rgb(0xb8, 0xbb, 0x26),  // bright green
        removal: rgb(0xfb, 0x49, 0x34),   // bright red
        change: rgb(0xfe, 0x80, 0x19),    // bright orange
        unchanged: rgb(0xa8, 0x99, 0x84), // gray (fg4)

        help: rgb(0x8e, 0xc0, 0x7c),    // bright aqua
        info: rgb(0x83, 0xa5, 0x98),    // bright blue
        warning: rgb(0xfa, 0xbd, 0x2f), // bright yellow
        error: rgb(0xfb, 0x49, 0x34),   // bright red
    };

    /// Gruvbox light.
    pub const LIGHT: Palette = Palette {
        text: rgb(0x3c, 0x38, 0x36),      // fg1
        highlight: rgb(0xb5, 0x76, 0x14), // faded yellow

        addition: rgb(0x79, 0x74, 0x0e),  // faded green
        removal: rgb(0x9d, 0x00, 0x06),   // faded red
        change: rgb(0xaf, 0x3a, 0x03),    // faded orange
        unchanged: rgb(0x7c, 0x6f, 0x64), // gray (fg4)

        help: rgb(0x42, 0x7b, 0x58),    // faded aqua
        info: rgb(0x07, 0x66, 0x78),    // faded blue
        warning: rgb(0xb5, 0x76, 0x14), // faded yellow
        error: rgb(0x9d, 0x00, 0x06),   // faded red
    };

    /// The preset palette for a theme.
    pub fn preset(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self::DARK,
            Theme::Light => Self::LIGHT,
        }
    }
}

/// Some content with a style attached.
///
/// Created via the [Stylize] methods. Formats as the plain content while
/// styled output is disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Styled<D> {
    content: D,
    style: ContentStyle,
}

impl<D> Styled<D> {
    /// The wrapped content.
    pub fn content(&self) -> &D {
        &self.content
    }

    /// The accumulated style.
    pub fn style(&self) -> &ContentStyle {
        &self.style
    }

    /// Set the foreground color directly.
    fn fg(mut self, color: Color) -> Self {
        self.style.foreground_color = Some(color);
        self
    }
}

impl<D: Display> Styled<D> {
    /// Format the content, with or without the style applied.
    fn fmt_styled(&self, f: &mut Formatter<'_>, styled: bool) -> fmt::Result {
        if styled {
            self.style.apply(&self.content).fmt(f)
        } else {
            self.content.fmt(f)
        }
    }
}

impl<D: Display> Display for Styled<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_styled(f, enabled())
    }
}

/// Generates styling methods for [`Styled`] and the [`Stylize`] trait, which acts as a conversion
/// helper for all types implementing `Display` into [`Styled`].
macro_rules! stylize_methods {
    (
        roles { $($role:ident),* $(,)? }
        attributes { $($attr_method:ident => $attribute:ident),* $(,)? }
    ) => {
        /// Styling methods available on everything that implements [Display].
        pub trait Stylize: Display + Sized {
            /// Wrap the value in a [`Styled`] struct.
            fn styled(self) -> Styled<Self> {
                Styled {
                    content: self,
                    style: ContentStyle::default(),
                }
            }

            $(
                #[doc = concat!("Color as `", stringify!($role), "` from the [`Palette`].")]
                fn $role(self) -> Styled<Self> {
                    self.styled().$role()
                }
            )*

            $(
                #[doc = concat!("Apply the `", stringify!($attribute), "` crossterm [`Attribute`].")]
                fn $attr_method(self) -> Styled<Self> {
                    self.styled().$attr_method()
                }
            )*
        }

        impl<D: Display> Stylize for D {}

        impl<D> Styled<D> {
            $(
                #[doc = concat!("Color as `", stringify!($role), "` from the [`Palette`].")]
                pub fn $role(self) -> Self {
                    self.fg(palette().$role)
                }
            )*

            $(
                #[doc = concat!("Apply the `", stringify!($attribute), "` crossterm [`Attribute`].")]
                pub fn $attr_method(mut self) -> Self {
                    self.style.attributes.set(Attribute::$attribute);
                    self
                }
            )*
        }
    };
}

stylize_methods! {
    roles { text, highlight, addition, removal, change, unchanged, help, info, warning, error }
    attributes { bold => Bold, dim => Dim, italic => Italic, underlined => Underlined }
}
