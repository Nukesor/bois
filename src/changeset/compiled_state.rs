use std::collections::{HashMap, HashSet};

use crate::{
    handlers::{packages::PackageManager, services::ServiceManager},
    state::{group::Group, host::Host, State},
};

/// This struct represents a rough compiled overview of all
/// services, packages and files that're contained in a state.
///
/// This simplified view on a state is used for simple diffing between old and new states to clean
/// up old state on the state.
#[derive(Default, Debug)]
pub struct CompiledState {
    /// A simple collection of **all** installed packages.
    /// We don't care where those packages come from, they just have to be requested by some file.
    pub deployed_packages: HashMap<PackageManager, HashSet<String>>,

    /// All services that've either been started and/or enabled
    pub enabled_services: HashMap<ServiceManager, HashSet<String>>,
    pub started_services: HashMap<ServiceManager, HashSet<String>>,
    // /// The tree of all files that're to be deployed for this state.
    // pub files: HashMap<PackageManager, String>,
}

impl CompiledState {
    pub fn from_state(state: &State) -> Self {
        let mut compiled_state = Self {
            deployed_packages: state.packages.clone(),
            ..Default::default()
        };

        handle_host(&mut compiled_state, &state.host);

        for group in state.host.groups.iter() {
            handle_group(&mut compiled_state, group);
        }

        compiled_state
    }
}

fn handle_host(_compiled_state: &mut CompiledState, _host: &Host) {}

fn handle_group(_compiled_state: &mut CompiledState, _group: &Group) {}
