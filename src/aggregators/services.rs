use std::collections::{BTreeMap, BTreeSet};

use log::debug;

use crate::{config::services::Service, state::ServiceManager};

/// Merge the host's and all traits' services.
pub fn aggregate_services(
    host_services: &BTreeMap<ServiceManager, BTreeSet<Service>>,
    traits: &[(String, crate::config::traits::TraitConfig)],
) -> BTreeMap<ServiceManager, BTreeSet<Service>> {
    let mut services = BTreeMap::new();

    merge_services("the host config", host_services, &mut services);
    for (name, trait_config) in traits {
        merge_services(
            &format!("the config of trait '{name}'"),
            &trait_config.services,
            &mut services,
        );
    }

    services
}

/// Merge new service declarations into the set of already-known services.
///
/// Service names are normalized first (e.g. `ntp` -> `ntp.service`), so different
/// spellings of the same unit are correctly deduplicated.
pub fn merge_services(
    source: &str,
    new: &BTreeMap<ServiceManager, BTreeSet<Service>>,
    known: &mut BTreeMap<ServiceManager, BTreeSet<Service>>,
) {
    for (manager, new_services) in new {
        let known_services = known.entry(*manager).or_default();

        for service in new_services {
            // Normalize the service name
            let service = service.clone().normalize(*manager);

            // Check if this is a duplicate
            let existing = known_services
                .iter()
                .find(|known| known.name == service.name)
                .cloned();

            let Some(existing) = existing else {
                // It's not. Add and continue
                known_services.insert(service);
                continue;
            };

            debug!("Found duplicate service '{}' in {source}", service.name);
            // Replace, if the the existing doesn't have `start` set.
            if service.start && !existing.start {
                known_services.remove(&existing);
                known_services.insert(service);
            }
        }
    }
}
