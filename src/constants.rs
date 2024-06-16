//! This module contains some runtime constants that are required for normal operation.
//!
//! Some of these are initialized at the start of the runtime and might panic.

use lazy_static::lazy_static;

lazy_static! {
    pub static ref CURRENT_USER: String = users::get_current_username()
        .expect("Couldn't read current unix username. Exiting.")
        .to_string_lossy()
        .to_string();
    pub static ref CURRENT_GROUP: String = users::get_current_groupname()
        .expect("Couldn't read current unix groupname. Exiting.")
        .to_string_lossy()
        .to_string();
}
