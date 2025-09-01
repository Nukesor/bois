use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::{info, trace};
use serde::{Deserialize, Serialize};

use super::directory::*;
use crate::constants::{CURRENT_GROUP, CURRENT_USER};
use crate::helper::expand_home;
use crate::state::file_parser::read_file;
use crate::templating::render_template;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Entry {
    File(File),
    Directory(Directory),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct File {
    /// The relative path to the source file.
    /// Relative to the root directory of the configuration (i.e. Host/Group directory).
    /// We need this information to determine the destination on the target file system.
    pub relative_path: PathBuf,

    /// The parsed configuration block for this file, if one exists.
    #[serde(default)]
    pub config: FileConfig,

    /// The actual configuration file's content, without the bois configuration block.
    pub content: String,
}

impl File {
    /// By default, the destination path calculates as follows.
    /// Default target directory (based on host/group/default) + relative path of this file from host/group.
    ///
    /// However, if a path override exists, we always use it.
    /// - If it's an absoulte path, we just use that path.
    ///   This can be used to deploy files **outside** the default target dir.
    /// - If it's a relative path, we just append it to the target_dir.
    pub fn file_path(&self, root: &Path) -> PathBuf {
        let mut path = if let Some(path) = &self.config.path() {
            if path.is_absolute() {
                path.clone()
            } else {
                root.join(path)
            }
        } else {
            root.join(&self.relative_path)
        };

        // If the a rename is requested, set the file name
        if let Some(file_name) = &self.config.rename {
            path.set_file_name(file_name);
        }

        path
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
    pub permissions: Option<u32>,

    /// Overwrite the templating delimiters used to start jinja blocks.
    /// See: <https://docs.rs/minijinja/latest/minijinja/syntax/struct.SyntaxConfig.html>
    pub delimiters: Option<Delimiters>,

    /// Whether this file should be treated as a template.
    /// Defaults to `false` to prevent unwanted behavior.
    #[serde(default)]
    pub template: bool,
}

impl FileConfig {
    pub fn path(&self) -> Option<PathBuf> {
        self.path.as_ref().map(|path| expand_home(path))
    }

    pub fn override_path(&mut self, path: PathBuf) {
        self.path = Some(path)
    }
}

/// Process a directory entry.
/// This function is a convenient wrapper that calls the `read_{directory|file}` functions.
/// In here we do some preparation, such as appending the name of the entry to the relative path
/// and the path_override, if applicable.
///
/// Params:
/// `root` The root of the bois configuration directory.
///        We need this to be able to read the file from the filesystem.
/// `entry` The actual file entry.
/// `directory` The representation of the directory we're currently processing.
///             All files/directories must be added to this `Directory`.
pub fn read_entry(
    root: &Path,
    relative_path: &Path,
    entry: DirEntry,
    directory: &mut Directory,
    mut path_override: Option<PathBuf>,
    template_vars: &serde_yaml::Value,
) -> Result<()> {
    let file_name = entry.file_name();

    let relative_path = relative_path.join(&file_name);

    // If there's an active override, adjust the override for the next level.
    if let Some(path) = path_override {
        path_override = Some(path.join(&file_name));
    }

    // Recursively discover new directories
    let path = entry.path();
    if path.is_dir() {
        let sub_directory = read_directory(root, &relative_path, path_override, template_vars)?;
        directory.entries.push(Entry::Directory(sub_directory));
    } else if path.is_file() {
        trace!("Reading file {path:?}");
        let mut file = read_file(root, &relative_path)?;

        // Check if there's an active path override from a parent directory.
        // If the file doesn't have its own override, use the one from the parent.
        if let Some(path_override) = path_override {
            if file.config.path().is_none() {
                file.config.override_path(path_override);
            }
        }

        // Perform templating, if enabled
        // Otherwise return the raw content.
        if file.config.template {
            info!("Starting templating for file {path:?}");
            file.content = render_template(&file.content, template_vars, &file.config.delimiters)
                .context(format!("Error for template at {path:?}"))?
        };

        directory.entries.push(Entry::File(file));
    }

    Ok(())
}

/// This impl block contains convenience getters for file metadata, which fall back to
/// default values.
impl FileConfig {
    pub fn permissions(&self) -> u32 {
        self.permissions.unwrap_or(0o644)
    }

    pub fn owner(&self) -> String {
        self.owner.clone().unwrap_or(CURRENT_USER.clone())
    }

    pub fn group(&self) -> String {
        self.group.clone().unwrap_or(CURRENT_GROUP.clone())
    }
}
