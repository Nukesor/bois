use std::collections::BTreeSet;

use anyhow::Result;

use crate::{
    config::bois::RunMode,
    state::{PackageManager, ServiceManager},
    system_state::{package_managers::SystemPackages, service_managers::SystemServices},
};

pub mod entry;
pub mod package_managers;
pub mod service_managers;

/// This state holds all important information about the system we're running on.
///
/// It's supposed to be passed around and updated while performing operations.
/// The idea is to minimize calls to external tools such as package managers or
/// systemd: every query result is cached.
///
/// File information is deliberately **not** part of this state. The filesystem
/// is queried live per path during comparisons, as we cannot pull the whole
/// filesystem into a state struct.
#[derive(Debug)]
pub struct SystemState {
    /// The mode bois runs in.
    /// - User mode: target the user's services
    /// - System mode: target system-wide services.
    mode: RunMode,
    packages: SystemPackages,
    services: SystemServices,
}

impl SystemState {
    pub fn new(mode: RunMode) -> Result<Self> {
        Ok(SystemState {
            mode,
            packages: SystemPackages::default(),
            services: SystemServices::default(),
        })
    }

    /// The mode bois runs in (user vs. system configuration).
    pub fn mode(&self) -> RunMode {
        self.mode
    }

    /// Get all installed packages for the current system, which includes potential dependencies.
    pub fn packages(&mut self, manager: PackageManager) -> Result<&BTreeSet<String>> {
        self.packages.packages(manager)
    }

    /// Get all **explicitly** installed packages for the current system.
    pub fn explicit_packages(&mut self, manager: PackageManager) -> Result<&BTreeSet<String>> {
        self.packages.explicit_packages(manager)
    }

    /// Re-query the installed packages, e.g. after packages were un-/installed.
    pub fn update_packages(&mut self, manager: PackageManager) -> Result<()> {
        self.packages.update_packages(manager)
    }

    /// Get the names of all package *groups* (e.g. pacman's `base-devel`) that
    /// exist on the system.
    pub fn package_groups(&mut self, manager: PackageManager) -> Result<&BTreeSet<String>> {
        self.packages.package_groups(manager)
    }

    /// Get the list of packages a package *group* consists of.
    pub fn packages_for_group(
        &mut self,
        manager: PackageManager,
        group: &str,
    ) -> Result<BTreeSet<String>> {
        self.packages.packages_for_group(manager, group)
    }

    /// Whether the given service is enabled.
    /// Services whose unit doesn't yet exist count as "not enabled".
    pub fn service_enabled(&mut self, manager: ServiceManager, name: &str) -> Result<bool> {
        self.services.is_enabled(manager, name, self.mode)
    }

    /// Whether the given service is currently running.
    pub fn service_active(&mut self, manager: ServiceManager, name: &str) -> Result<bool> {
        self.services.is_active(manager, name, self.mode)
    }

    /// Update a service's cached enabled state.
    pub fn update_service_enabled(&mut self, manager: ServiceManager, name: &str, enabled: bool) {
        self.services.update_enabled(manager, name, enabled)
    }

    /// Update a service's cached active state.
    pub fn update_service_active(&mut self, manager: ServiceManager, name: &str, active: bool) {
        self.services.update_active(manager, name, active)
    }
}
