use std::collections::{HashMap, HashSet};

use crate::{
    handlers::{packages::PackageManager, services::ServiceManager},
    state::{host::Host, State},
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
    pub fn from_state(state: State) -> Self {
        let mut compiled_state = Self::default();

        handle_host(&mut compiled_state, &state.host);

        for group in state.host.groups.iter() {
            handle_group(&mut compiled_state, &state.host);
        }

        compiled_state
    }
}

fn handle_host(compiled_state: &mut CompiledState, host: &Host) {
    // Merge all packages from the host config file into the compiled packages lists.
    for (manager, packages) in host.config.packages.iter() {
        let compiled_packages = compiled_state
            .deployed_packages
            .entry(*manager)
            .or_default();

        compiled_packages.extend(packages.clone().into_iter());
    }
}

fn handle_group(compiled_state: &mut CompiledState, host: &Host) {
    // Merge all packages from the host config file into the compiled packages lists.
    for (manager, packages) in host.config.packages.iter() {
        let compiled_packages = compiled_state
            .deployed_packages
            .entry(*manager)
            .or_default();

        compiled_packages.extend(packages.clone().into_iter());
    }
}
