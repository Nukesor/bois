//! This module contains some runtime constants that are required for normal operation.
//!
//! Some of these are initialized at the start of the runtime and might panic.

use lazy_static::lazy_static;
use nix::unistd::{Gid, Group as NixGroup, Uid, User as NixUser};

lazy_static! {
    pub static ref CURRENT_USER: String = NixUser::from_uid(Uid::current())
        .expect("Couldn't read current user. Exiting.")
        .expect("Couldn't find user for current id. Exiting.")
        .name;
    pub static ref CURRENT_GROUP: String = NixGroup::from_gid(Gid::current())
        .expect("Couldn't read current group. Exiting.")
        .expect("Couldn't find group for current id. Exiting.")
        .name;
}
