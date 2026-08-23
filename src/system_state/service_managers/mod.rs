use std::collections::BTreeMap;

use anyhow::Result;

use crate::{config::bois::RunMode, state::ServiceManager};

mod systemd;

/// A service manager state cache.
///
/// Each unit's state is only queried once and cached afterwards, so we don't
/// repeatedly shell out to service managers during a single run.
#[derive(Debug, Default)]
pub struct SystemServices {
    enabled: BTreeMap<ServiceManager, BTreeMap<String, bool>>,
    active: BTreeMap<ServiceManager, BTreeMap<String, bool>>,
}

impl SystemServices {
    /// Whether the given service is enabled or doesn't need enabling, such as
    /// static systemd units.
    ///
    /// Services whose unit doesn't exist count as "not enabled".
    pub fn is_enabled(
        &mut self,
        manager: ServiceManager,
        name: &str,
        mode: RunMode,
    ) -> Result<bool> {
        let cache = self.enabled.entry(manager).or_default();
        if let Some(enabled) = cache.get(name) {
            return Ok(*enabled);
        }

        let enabled = match manager {
            ServiceManager::Systemd => systemd::is_enabled(name, mode)?,
        };
        cache.insert(name.to_string(), enabled);

        Ok(enabled)
    }

    /// Whether the given service is currently running.
    pub fn is_active(
        &mut self,
        manager: ServiceManager,
        name: &str,
        mode: RunMode,
    ) -> Result<bool> {
        let cache = self.active.entry(manager).or_default();
        if let Some(active) = cache.get(name) {
            return Ok(*active);
        }

        let active = match manager {
            ServiceManager::Systemd => systemd::is_active(name, mode)?,
        };
        cache.insert(name.to_string(), active);

        Ok(active)
    }

    /// Update a service's cached enabled state.
    pub fn update_enabled(&mut self, manager: ServiceManager, name: &str, enabled: bool) {
        self.enabled
            .entry(manager)
            .or_default()
            .insert(name.to_string(), enabled);
    }

    /// Update a service's cached active state.
    pub fn update_active(&mut self, manager: ServiceManager, name: &str, active: bool) {
        self.active
            .entry(manager)
            .or_default()
            .insert(name.to_string(), active);
    }
}
