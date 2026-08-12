use std::collections::BTreeMap;

use anyhow::Result;

use crate::{
    changeset::{ServiceDisable, ServiceEnable},
    state::ServiceManager,
    system_state::SystemState,
};

pub mod systemd;

/// Execute a list of service enables.
///
/// Services are grouped by service manager and then enabled in one go.
pub fn enable_services(system_state: &mut SystemState, services: &[ServiceEnable]) -> Result<()> {
    let mut sorted_services: BTreeMap<ServiceManager, Vec<&ServiceEnable>> = BTreeMap::new();

    // First up, sort all services by manager.
    for service in services {
        let list = sorted_services.entry(service.manager).or_default();
        list.push(service);
    }

    for (manager, services) in sorted_services {
        match manager {
            ServiceManager::Systemd => systemd::enable_services(system_state, &services)?,
        }
    }

    Ok(())
}

/// Execute a list of service disables. The services are stopped and disabled.
///
/// Services are grouped by service manager and then disabled in one go.
pub fn disable_services(system_state: &mut SystemState, services: &[ServiceDisable]) -> Result<()> {
    let mut sorted_services: BTreeMap<ServiceManager, Vec<&ServiceDisable>> = BTreeMap::new();

    // First up, sort all services by manager.
    for service in services {
        let list = sorted_services.entry(service.manager).or_default();
        list.push(service);
    }

    for (manager, services) in sorted_services {
        match manager {
            ServiceManager::Systemd => systemd::disable_services(system_state, &services)?,
        }
    }

    Ok(())
}
