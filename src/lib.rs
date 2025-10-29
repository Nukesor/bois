use std::sync::OnceLock;

pub mod args;
pub mod changeset;
pub mod commands;
pub mod config;
pub mod constants;
pub mod error;
pub mod handlers;
pub mod password_managers;
pub mod state;
pub mod system_state;
pub mod templating;
pub mod ui;

use config::bois::Configuration;

/// Expose the config as a global.
/// This is somewhat of an antipatter, but is needed to access the configuration inside of
/// minijinja custom filters/functions. We have no way to pass additional arguments to those, as
/// they're called by minijinja. (We could maybe use closures when registering the minijinja
/// functions, but that feels also somewhat ugly.)
///
/// Avoid to use this anywhere outside of minijinja's filters/functions.
pub static CONFIG: OnceLock<Configuration> = OnceLock::new();
