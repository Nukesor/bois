//! The handler functions used during deployemnt and cleanup.
//!
//! Handlers take the operations of a [crate::changeset::Changeset] and apply
//! them to the system. All comparison logic lives in [crate::changeset].

pub mod packages;
pub mod paths;
pub mod services;
