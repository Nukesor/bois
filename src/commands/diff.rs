use anyhow::Result;
use log::trace;

use crate::{config::Configuration, state::State, system_state::SystemState};

pub fn diff(config: Configuration) -> Result<()> {
    diff_packages(&config)
}

/// Compare the current desired state with the current state of the system.
/// Show any packages that are installed on the system, but aren't tracked by us.
fn diff_packages(config: &Configuration) -> Result<()> {
    let mut system_state = SystemState::new()?;
    trace!("System state: {system_state:#?}");

    // Read the current desired system state from the files in the specified bois directory.
    let desired_state = State::new(config, &mut system_state)?;
    trace!("Config state: {desired_state:#?}");

    for (manager, packages) in desired_state.packages {
        // Get all untracked packages in a sorted list
        let installed_packages = system_state.installed_packages(manager)?;
        let mut untracked_packages: Vec<String> = installed_packages
            .into_iter()
            .filter(|pkg| !packages.contains(pkg))
            .collect();
        untracked_packages.sort();

        // Format the strings a bit, so the output is nice.
        untracked_packages = untracked_packages
            .into_iter()
            .map(|pkg| format!("- {pkg}"))
            .collect();

        println!(
            "Untracked packages on system for manager {manager}:\n{}",
            untracked_packages.join("\n")
        );
    }

    Ok(())
}
