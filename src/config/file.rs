use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::helper::expand_home;
use crate::constants::{CURRENT_GROUP, CURRENT_USER};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FileConfig {
    /// If this is set, this path will be used as a destination.
    /// If it's an relative path, it'll be treated as relative to the default target directory.
    /// If it's an absolute path, that absolute path will be used.
    path: Option<PathBuf>,
    /// Use this option to override the filename.
    /// Useful to have configs live as normal files in the bois directory, even though they need to
    /// later become '.' dotfiles.
    pub rename: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
    /// This is represented as a octal `Oo640` in yaml.
    /// It's automatically parsed to a u32, which can then be used by the std lib.
    pub mode: Option<u32>,

    /// Overwrite the templating delimiters used to start jinja blocks.
    /// See: <https://docs.rs/minijinja/latest/minijinja/syntax/struct.SyntaxConfig.html>
    pub delimiters: Option<Delimiters>,

    /// Whether this file should be treated as a template.
    /// Defaults to `false` to prevent unwanted behavior.
    #[serde(default)]
    pub template: bool,
}

/// This impl block contains convenience getters for file metadata, which fall back to
/// default values.
impl FileConfig {
    pub fn path(&self) -> Option<PathBuf> {
        self.path.as_ref().map(|path| expand_home(path))
    }

    pub fn override_path(&mut self, path: PathBuf) {
        self.path = Some(path)
    }

    pub fn owner(&self) -> String {
        self.owner.clone().unwrap_or(CURRENT_USER.clone())
    }

    pub fn group(&self) -> String {
        self.group.clone().unwrap_or(CURRENT_GROUP.clone())
    }
}

/// Overwrite the templating delimiters used to start jinja blocks.
/// See: <https://docs.rs/minijinja/latest/minijinja/syntax/struct.SyntaxConfig.html>
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Delimiters {
    /// Use this to prefix the start element of each delimiter type with this tring.
    ///
    /// This is useful to prevent clashes with in-file syntax.
    /// E.g. the `#` prefix will make the templating appear as a comment for
    /// all languages that interpret `#` as a comment.
    ///
    /// That way, formatter, language servers and such won't produce errors.
    ///
    /// Example:
    /// ```j2
    /// #{% if host == "my_machine" }%
    /// ...
    /// other stuff here
    /// ...
    /// #{% endif %}
    /// ```
    #[serde(default)]
    pub prefix: Option<String>,
    /// The delimiters used to define a logical block.
    /// E.g. `("{%", "%}")`
    #[serde(default = "Delimiters::default_block_delimiter")]
    pub block: (String, String),
    /// The delimiters used to define a variable block.
    /// E.g. `("{{", "}}")`
    #[serde(default = "Delimiters::default_variable_delimiter")]
    pub variable: (String, String),
    /// The delimiters used to define a comment block.
    /// E.g. `("{#", "#}")`
    #[serde(default = "Delimiters::default_comment_delimiter")]
    pub comment: (String, String),
}

impl Delimiters {
    fn default_block_delimiter() -> (String, String) {
        ("{%".to_string(), "%}".to_string())
    }

    fn default_variable_delimiter() -> (String, String) {
        ("{{".to_string(), "}}".to_string())
    }

    fn default_comment_delimiter() -> (String, String) {
        ("{#".to_string(), "#}".to_string())
    }

    /// Get the jinja `block` syntax to use.
    ///
    /// If a prefix is set, apply it to the start element.
    pub fn block(&self) -> (String, String) {
        if let Some(mut prefix) = self.prefix.clone() {
            prefix.push_str(&self.block.0);
            return (prefix, self.block.1.clone());
        }

        self.block.clone()
    }

    /// Get the jinja `variable` syntax to use.
    ///
    /// If a prefix is set, apply it to the start element.
    pub fn variable(&self) -> (String, String) {
        if let Some(mut prefix) = self.prefix.clone() {
            prefix.push_str(&self.variable.0);
            return (prefix, self.variable.1.clone());
        }

        self.variable.clone()
    }

    /// Get the jinja `comment` syntax to use.
    ///
    /// If a prefix is set, apply it to the start element.
    pub fn comment(&self) -> (String, String) {
        if let Some(mut prefix) = self.prefix.clone() {
            prefix.push_str(&self.comment.0);
            return (prefix, self.comment.1.clone());
        }

        self.comment.clone()
    }
}
