//! The `services` configuration section, shared by host, trait and directory
//! configuration files.

use serde::{Deserialize, Serialize};

use crate::state::ServiceManager;

/// A single service that should be enabled on the system.
///
/// A service can be declared as a plain string or, to set additional options,
/// as a map:
///
/// ```yaml
/// services:
///   systemd:
///     - backup.timer
///     - name: docker
///       start: true
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(from = "ServiceDeclaration")]
pub struct Service {
    /// The name of the service unit.
    pub name: String,
    /// Whether the service should also be started right away at the moment it
    /// gets enabled. Services that're already enabled are never started.
    pub start: bool,
}

/// The two YAML representations that're accepted for a [Service].
#[derive(Deserialize)]
#[serde(untagged)]
enum ServiceDeclaration {
    /// A plain service name. The service will only be enabled, not started.
    Name(String),
    Options {
        name: String,
        #[serde(default)]
        start: bool,
    },
}

impl From<ServiceDeclaration> for Service {
    fn from(declaration: ServiceDeclaration) -> Self {
        match declaration {
            ServiceDeclaration::Name(name) => Service { name, start: false },
            ServiceDeclaration::Options { name, start } => Service { name, start },
        }
    }
}

/// All unit suffixes that're recognized by systemd.
const SYSTEMD_UNIT_SUFFIXES: [&str; 11] = [
    ".service",
    ".socket",
    ".device",
    ".mount",
    ".automount",
    ".swap",
    ".target",
    ".path",
    ".timer",
    ".slice",
    ".scope",
];

impl Service {
    /// Normalize the service name for the given service manager.
    ///
    /// - Systemd interprets unit names without a unit suffix as services (e.g. `ntp` and
    ///   `ntp.service` are the same).
    ///
    /// This function normalizes any such issues to avoid ambiguities.
    pub fn normalize(mut self, manager: ServiceManager) -> Service {
        match manager {
            ServiceManager::Systemd => {
                let has_suffix = SYSTEMD_UNIT_SUFFIXES
                    .iter()
                    .any(|suffix| self.name.ends_with(suffix));
                if !has_suffix {
                    self.name = format!("{}.service", self.name);
                }
            }
        }

        self
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    /// Unit names without a unit suffix are normalized to explicit `.service`
    /// units. Names with a suffix stay untouched.
    #[rstest]
    #[case("ntp", "ntp.service")]
    #[case("ntp.service", "ntp.service")]
    // Dots in unit names don't count as suffix delimiters.
    #[case(
        "dbus-org.freedesktop.timesync1",
        "dbus-org.freedesktop.timesync1.service"
    )]
    fn test_systemd_service_name_normalization(#[case] input: &str, #[case] expected: &str) {
        let service = Service {
            name: input.to_string(),
            start: false,
        };
        assert_eq!(service.normalize(ServiceManager::Systemd).name, expected);
    }
}
