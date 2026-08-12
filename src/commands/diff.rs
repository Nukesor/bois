use anyhow::Result;
use log::trace;

use crate::{aggregators::aggregate_state, config::bois::Configuration, system_state::SystemState};

pub fn diff(config: Configuration) -> Result<()> {
    diff_packages(&config)
}

/// Compare the current desired state with the current state of the system.
/// Show any packages that are installed on the system, but aren't tracked by us.
fn diff_packages(config: &Configuration) -> Result<()> {
    let mut system_state = SystemState::new(config.mode)?;

    // Read the current desired system state from the files in the specified bois directory.
    let desired_state = aggregate_state(config, &mut system_state)?;
    trace!("Config state: {desired_state:#?}");

    let mut untracked_changes_exist = false;
    for (manager, packages) in desired_state.packages {
        // Get all explicitly installed packages.
        // We don't want to consider dependencies, as they're not important.
        let installed_packages = system_state.explicit_packages(manager)?;

        // Now filter all packages that are specified in the bois configuration.
        // BTreeSets iterate in order, so the output is deterministic.
        let untracked_packages: Vec<String> = installed_packages
            .iter()
            .filter(|pkg| !packages.contains(*pkg))
            .map(|pkg| format!("- {pkg}"))
            .collect();

        // Continue if there're no untracked packages.
        if untracked_packages.is_empty() {
            continue;
        }
        untracked_changes_exist = true;

        println!(
            "Untracked packages on system for manager {manager}:\n{}",
            untracked_packages.join("\n")
        );
    }

    if !untracked_changes_exist {
        println!("Packages: match boi's definition")
    }

    Ok(())
}
